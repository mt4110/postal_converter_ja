package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// Styles
var (
	titleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("#FAFAFA")).
			Background(lipgloss.Color("#7D56F4")).
			Padding(1, 2).
			MarginBottom(1)

	itemStyle = lipgloss.NewStyle().
			PaddingLeft(2)

	selectedItemStyle = lipgloss.NewStyle().
				PaddingLeft(2).
				Foreground(lipgloss.Color("170"))

	disabledItemStyle = lipgloss.NewStyle().
				PaddingLeft(2).
				Foreground(lipgloss.Color("#626262"))

	checkMark = lipgloss.NewStyle().Foreground(lipgloss.Color("42")).SetString("✓")

	quitStyle = lipgloss.NewStyle().
			MarginTop(1).
			Foreground(lipgloss.Color("#626262"))

	guideStyle = lipgloss.NewStyle().
			MarginTop(1).
			Foreground(lipgloss.Color("#00D7FF")).
			Italic(true)
)

type status int

const (
	statusPending status = iota
	statusDone
)

type model struct {
	choices  []string
	cursor   int
	quitting bool

	// State
	dbStatus       status
	crawlerStatus  status
	apiStatus      status
	frontendStatus status

	msg string // Status message
}

const (
	apiBaseURL   = "http://127.0.0.1:3202"
	swaggerURL   = "http://127.0.0.1:3202/docs"
	frontendURL  = "http://127.0.0.1:3203"
)

func initialModel() model {
	return model{
		choices: []string{
			"🚀 Start Databases (Docker)",
			"🕷️  Start Crawler (New Terminal)",
			"🔌 Start API Server (New Terminal)",
			"💻 Start Frontend (New Terminal)",
			"🛑 Stop Databases",
			"🚪 Exit",
		},
		dbStatus:       statusPending,
		crawlerStatus:  statusPending,
		apiStatus:      statusPending,
		frontendStatus: statusPending,
		msg:            "Please start the Databases first.",
	}
}

func (m model) Init() tea.Cmd {
	return nil
}

// Custom messages for state updates
type dbStartedMsg struct{ err error }
type crawlerStartedMsg struct{ err error }
type apiStartedMsg struct{ err error }
type frontendStartedMsg struct{ err error }
type dbStoppedMsg struct{ err error }

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		return m.handleKeyMsg(msg)
	case dbStartedMsg:
		return m.handleDbStarted(msg)
	case crawlerStartedMsg:
		return m.handleCrawlerStarted(msg)
	case apiStartedMsg:
		return m.handleApiStarted(msg)
	case frontendStartedMsg:
		return m.handleFrontendStarted(msg)
	case dbStoppedMsg:
		return m.handleDbStopped(msg)
	}
	return m, nil
}

func (m model) handleKeyMsg(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch msg.String() {
	case "ctrl+c", "q":
		m.quitting = true
		return m, tea.Quit
	case "up", "k":
		if m.cursor > 0 {
			m.cursor--
		}
	case "down", "j":
		if m.cursor < len(m.choices)-1 {
			m.cursor++
		}
	case "enter", " ":
		return m.executeSelection(m.cursor)
	}
	return m, nil
}

func (m model) handleDbStarted(msg dbStartedMsg) (tea.Model, tea.Cmd) {
	if msg.err != nil {
		m.msg = fmt.Sprintf("Error starting DB: %v", msg.err)
	} else {
		m.dbStatus = statusDone
		m.msg = "Databases started! Now you can start Crawler or API."
		if m.cursor == 0 {
			m.cursor = 1
		}
	}
	return m, nil
}

func (m model) handleCrawlerStarted(msg crawlerStartedMsg) (tea.Model, tea.Cmd) {
	if msg.err != nil {
		m.msg = fmt.Sprintf("Error starting Crawler: %v", msg.err)
	} else {
		m.crawlerStatus = statusDone
		m.msg = "Crawler terminal opened."
	}
	return m, nil
}

func (m model) handleApiStarted(msg apiStartedMsg) (tea.Model, tea.Cmd) {
	if msg.err != nil {
		m.msg = fmt.Sprintf("Error starting API: %v", msg.err)
	} else {
		m.apiStatus = statusDone
		m.msg = fmt.Sprintf("API terminal opened. Swagger: %s", swaggerURL)
		if m.cursor == 2 {
			m.cursor = 3
		}
	}
	return m, nil
}

func (m model) handleFrontendStarted(msg frontendStartedMsg) (tea.Model, tea.Cmd) {
	if msg.err != nil {
		m.msg = fmt.Sprintf("Error starting Frontend: %v", msg.err)
	} else {
		m.frontendStatus = statusDone
		m.msg = fmt.Sprintf("Frontend terminal opened: %s", frontendURL)
	}
	return m, nil
}

func (m model) handleDbStopped(msg dbStoppedMsg) (tea.Model, tea.Cmd) {
	if msg.err != nil {
		m.msg = fmt.Sprintf("Error stopping DB: %v", msg.err)
	} else {
		m.dbStatus = statusPending
		m.crawlerStatus = statusPending
		m.apiStatus = statusPending
		m.frontendStatus = statusPending
		m.msg = "Databases stopped. Resetting state."
		m.cursor = 0
	}
	return m, nil
}

func (m model) executeSelection(index int) (tea.Model, tea.Cmd) {
	switch index {
	case 0: // Start Databases
		m.msg = "Starting databases..."
		return m, func() tea.Msg {
			err := runCommand("docker-compose", "up", "-d")
			return dbStartedMsg{err}
		}
	case 1: // Start Crawler
		if m.dbStatus != statusDone {
			m.msg = "⚠️  Please start Databases first!"
			return m, nil
		}
		m.msg = "Opening Crawler terminal..."
		return m, func() tea.Msg {
			projectRoot := projectRootDir()
			err := openInTerminal(
				fmt.Sprintf("cd '%s' && nix develop --command bash -lc 'cd worker/crawler && cargo run --release --bin crawler'", projectRoot),
			)
			time.Sleep(500 * time.Millisecond)
			return crawlerStartedMsg{err}
		}
	case 2: // Start API
		if m.dbStatus != statusDone {
			m.msg = "⚠️  Please start Databases first!"
			return m, nil
		}
		m.msg = "Opening API terminal..."
		return m, func() tea.Msg {
			projectRoot := projectRootDir()
			err := openInTerminal(
				fmt.Sprintf("cd '%s' && nix develop --command bash -lc 'cd worker/api && cargo run --release --bin api'", projectRoot),
			)
			time.Sleep(500 * time.Millisecond)
			return apiStartedMsg{err}
		}
	case 3: // Start Frontend
		if m.apiStatus != statusDone {
			m.msg = "⚠️  Please start API first!"
			return m, nil
		}
		m.msg = "Opening Frontend terminal..."
		return m, func() tea.Msg {
			projectRoot := projectRootDir()
			err := openInTerminal(
				fmt.Sprintf("cd '%s' && nix develop --command bash -lc 'cd frontend && yarn install && yarn dev'", projectRoot),
			)
			time.Sleep(500 * time.Millisecond)
			return frontendStartedMsg{err}
		}
	case 4: // Stop Databases
		m.msg = "Stopping databases..."
		return m, func() tea.Msg {
			err := runCommand("docker-compose", "down")
			return dbStoppedMsg{err}
		}
	case 5: // Exit
		m.quitting = true
		return m, tea.Quit
	}
	return m, nil
}

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func openInTerminal(command string) error {
	// macOS specific: Open a new Terminal window and run the command
	safeCommand := strings.ReplaceAll(command, "\"", "\\\"")
	script := fmt.Sprintf("tell application \"Terminal\" to do script \"%s\"", safeCommand)
	cmd := exec.Command("osascript", "-e", script)
	return cmd.Run()
}

func projectRootDir() string {
	cwd, err := os.Getwd()
	if err != nil {
		return "."
	}
	if filepath.Base(cwd) == "launcher" {
		return filepath.Dir(cwd)
	}
	return cwd
}

func (m model) View() string {
	if m.quitting {
		return quitStyle.Render("Bye! 👋")
	}

	s := titleStyle.Render("📮 Postal Converter JA Launcher") + "\n\n"

	for i, choice := range m.choices {
		s += m.renderItem(i, choice)
	}

	s += guideStyle.Render("\n" + m.msg + "\n")
	s += guideStyle.Render(fmt.Sprintf("API: %s | Swagger: %s | Frontend: %s\n", apiBaseURL, swaggerURL, frontendURL))
	s += quitStyle.Render("\nPress q to quit.\n")
	return s
}

func (m model) renderItem(i int, choice string) string {
	cursor := " "
	if m.cursor == i {
		cursor = "👉"
	}

	isEnabled, isDone := m.getItemStatus(i)

	label := choice
	if isDone {
		label = fmt.Sprintf("%s %s", choice, checkMark)
	}

	if !isEnabled {
		return disabledItemStyle.Render(fmt.Sprintf("%s %s (Locked 🔒)", cursor, choice)) + "\n"
	} else if m.cursor == i {
		return selectedItemStyle.Render(fmt.Sprintf("%s %s", cursor, label)) + "\n"
	} else {
		return itemStyle.Render(fmt.Sprintf("%s %s", cursor, label)) + "\n"
	}
}

func (m model) getItemStatus(i int) (bool, bool) {
	isEnabled := true
	isDone := false

	switch i {
	case 1, 2: // Crawler, API require DB
		if m.dbStatus != statusDone {
			isEnabled = false
		}
		if (i == 1 && m.crawlerStatus == statusDone) || (i == 2 && m.apiStatus == statusDone) {
			isDone = true
		}
	case 3: // Frontend requires API
		if m.apiStatus != statusDone {
			isEnabled = false
		}
		if m.frontendStatus == statusDone {
			isDone = true
		}
	case 0: // DB
		if m.dbStatus == statusDone {
			isDone = true
		}
	}
	return isEnabled, isDone
}

func main() {
	p := tea.NewProgram(initialModel())
	if _, err := p.Run(); err != nil {
		fmt.Printf("Alas, there's been an error: %v", err)
		os.Exit(1)
	}
}

package main

import (
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"
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
	apiCmd         *exec.Cmd
	frontendCmd    *exec.Cmd

	msg string // Status message
}

const (
	apiBaseURL  = "http://127.0.0.1:3202"
	swaggerURL  = "http://127.0.0.1:3202/docs"
	frontendURL = "http://127.0.0.1:3203"
	menuStartY  = 6
	apiLogPath  = "/tmp/postal_converter_ja_api.log"
	feLogPath   = "/tmp/postal_converter_ja_frontend.log"
)

func initialModel() model {
	return model{
		choices: []string{
			"🚀 Start Databases (Docker)",
			"🕷️  Start Crawler (New Terminal)",
			"🔌 Start API Server",
			"💻 Start Frontend",
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
type apiStartedMsg struct {
	err error
	cmd *exec.Cmd
}
type frontendStartedMsg struct {
	err error
	cmd *exec.Cmd
}
type dbStoppedMsg struct{ err error }

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		return m.handleKeyMsg(msg)
	case tea.MouseMsg:
		return m.handleMouseMsg(msg)
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
	case "1", "2", "3", "4", "5", "6":
		index := int(msg.String()[0]-'1')
		if index >= 0 && index < len(m.choices) {
			m.cursor = index
			return m.executeSelection(index)
		}
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

func (m model) handleMouseMsg(msg tea.MouseMsg) (tea.Model, tea.Cmd) {
	switch msg.Button {
	case tea.MouseButtonWheelUp:
		if m.cursor > 0 {
			m.cursor--
		}
		return m, nil
	case tea.MouseButtonWheelDown:
		if m.cursor < len(m.choices)-1 {
			m.cursor++
		}
		return m, nil
	case tea.MouseButtonLeft:
		if msg.Action != tea.MouseActionRelease {
			return m, nil
		}
		if idx, ok := menuIndexFromY(msg.Y, len(m.choices)); ok {
			m.cursor = idx
			return m.executeSelection(idx)
		}
		// Fallback: run current selection when click position is outside item rows.
		return m.executeSelection(m.cursor)
	default:
		return m, nil
	}
}

func menuIndexFromY(y int, count int) (int, bool) {
	idx := y - menuStartY
	if idx < 0 || idx >= count {
		return 0, false
	}
	return idx, true
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
		m.apiCmd = msg.cmd
		m.msg = fmt.Sprintf("API started. Swagger: %s (log: %s)", swaggerURL, apiLogPath)
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
		m.frontendCmd = msg.cmd
		m.msg = fmt.Sprintf("Frontend started: %s (log: %s)", frontendURL, feLogPath)
	}
	return m, nil
}

func (m model) handleDbStopped(msg dbStoppedMsg) (tea.Model, tea.Cmd) {
	if msg.err != nil {
		m.msg = fmt.Sprintf("Error stopping DB: %v", msg.err)
	} else {
		killManagedProcess(m.apiCmd)
		killManagedProcess(m.frontendCmd)
		m.dbStatus = statusPending
		m.crawlerStatus = statusPending
		m.apiStatus = statusPending
		m.frontendStatus = statusPending
		m.apiCmd = nil
		m.frontendCmd = nil
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
			err := runDockerCompose("--profile", "cache", "up", "-d", "postgres", "redis")
			if err == nil {
				err = waitForPostgresReady(90 * time.Second)
			}
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
				fmt.Sprintf(
					"cd '%s' && nix develop --command bash -lc 'cd worker/crawler && DATABASE_TYPE=postgres POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@127.0.0.1:3205/zip_code_db REDIS_URL=redis://127.0.0.1:3206 cargo run --release --bin crawler'",
					projectRoot,
				),
			)
			time.Sleep(500 * time.Millisecond)
			return crawlerStartedMsg{err}
		}
	case 2: // Start API
		if m.dbStatus != statusDone {
			m.msg = "⚠️  Please start Databases first!"
			return m, nil
		}
		m.msg = "Starting API process..."
		return m, func() tea.Msg {
			if isURLReady(swaggerURL, 2*time.Second) {
				return apiStartedMsg{cmd: m.apiCmd}
			}
			projectRoot := projectRootDir()
			cmd, err := startManagedProcess(
				fmt.Sprintf(
					"cd '%s' && nix develop --command bash -lc 'cd worker/api && DATABASE_TYPE=postgres POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@127.0.0.1:3205/zip_code_db REDIS_URL=redis://127.0.0.1:3206 cargo run --release --bin api'",
					projectRoot,
				),
				apiLogPath,
			)
			if err == nil {
				err = waitForURLOrProcessExit(cmd, swaggerURL, 90*time.Second)
			}
			if err != nil {
				return apiStartedMsg{err: fmt.Errorf("%w (log: %s)", err, apiLogPath)}
			}
			return apiStartedMsg{cmd: cmd}
		}
	case 3: // Start Frontend
		if m.apiStatus != statusDone {
			m.msg = "⚠️  Please start API first!"
			return m, nil
		}
		m.msg = "Starting Frontend process..."
		return m, func() tea.Msg {
			if isURLReady(frontendURL, 2*time.Second) {
				return frontendStartedMsg{cmd: m.frontendCmd}
			}
			projectRoot := projectRootDir()
			cmd, err := startManagedProcess(
				fmt.Sprintf(
					"cd '%s' && nix develop --command bash -lc 'cd frontend && if [ ! -d node_modules ]; then yarn install --frozen-lockfile || yarn install; fi && yarn dev'",
					projectRoot,
				),
				feLogPath,
			)
			if err == nil {
				err = waitForURLOrProcessExit(cmd, frontendURL, 90*time.Second)
			}
			if err != nil {
				return frontendStartedMsg{err: fmt.Errorf("%w (log: %s)", err, feLogPath)}
			}
			return frontendStartedMsg{cmd: cmd}
		}
	case 4: // Stop Databases
		m.msg = "Stopping databases..."
		return m, func() tea.Msg {
			err := runDockerCompose("--profile", "cache", "down")
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

func runDockerCompose(args ...string) error {
	if err := runCommand("docker", append([]string{"compose"}, args...)...); err == nil {
		return nil
	}
	return runCommand("docker-compose", args...)
}

func startManagedProcess(command string, logPath string) (*exec.Cmd, error) {
	logFile, err := os.OpenFile(logPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o644)
	if err != nil {
		return nil, err
	}

	cmd := exec.Command("bash", "-lc", command)
	cmd.Stdout = logFile
	cmd.Stderr = logFile
	if err := cmd.Start(); err != nil {
		_ = logFile.Close()
		return nil, err
	}
	_ = logFile.Close()
	return cmd, nil
}

func killManagedProcess(cmd *exec.Cmd) {
	if cmd == nil || cmd.Process == nil {
		return
	}
	_ = cmd.Process.Kill()
}

func isURLReady(url string, timeout time.Duration) bool {
	client := &http.Client{Timeout: timeout}
	resp, err := client.Get(url)
	if err != nil {
		return false
	}
	_ = resp.Body.Close()
	return resp.StatusCode >= 200 && resp.StatusCode < 500
}

func isProcessAlive(cmd *exec.Cmd) bool {
	if cmd == nil || cmd.Process == nil {
		return false
	}
	if cmd.ProcessState != nil && cmd.ProcessState.Exited() {
		return false
	}
	return cmd.Process.Signal(syscall.Signal(0)) == nil
}

func waitForURLOrProcessExit(cmd *exec.Cmd, url string, timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if isURLReady(url, 2*time.Second) {
			return nil
		}
		if !isProcessAlive(cmd) {
			return fmt.Errorf("process exited before service became ready at %s", url)
		}
		time.Sleep(1 * time.Second)
	}
	return fmt.Errorf("service did not become ready at %s within %s", url, timeout)
}

func waitForPostgresReady(timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		cmd := exec.Command("docker", "exec", "postgres_container", "pg_isready", "-U", "postgres", "-d", "zip_code_db")
		if err := cmd.Run(); err == nil {
			return nil
		}
		time.Sleep(2 * time.Second)
	}
	return fmt.Errorf("postgres_container was not ready within %s", timeout)
}

func waitForURL(url string, timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	client := &http.Client{Timeout: 2 * time.Second}
	for time.Now().Before(deadline) {
		resp, err := client.Get(url)
		if err == nil {
			_ = resp.Body.Close()
			if resp.StatusCode >= 200 && resp.StatusCode < 500 {
				return nil
			}
		}
		time.Sleep(1 * time.Second)
	}
	return fmt.Errorf("service did not become ready at %s within %s", url, timeout)
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
	p := tea.NewProgram(initialModel(), tea.WithMouseCellMotion())
	if _, err := p.Run(); err != nil {
		fmt.Printf("Alas, there's been an error: %v", err)
		os.Exit(1)
	}
}

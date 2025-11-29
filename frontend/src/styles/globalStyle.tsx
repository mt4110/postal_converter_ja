import { createGlobalStyle, DefaultTheme } from 'styled-components';

interface CustomTheme extends DefaultTheme {
  background: string;
  color: string;
}

const GlobalStyle = createGlobalStyle<{ theme: CustomTheme }>`
  body {
    background-color: ${(props) => props.theme.background};
    color: ${(props) => props.theme.color};
    font-family: 'Inter', sans-serif;
  }
`;

export default GlobalStyle;
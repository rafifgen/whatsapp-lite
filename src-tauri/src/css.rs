pub const CUSTOM_CSS: &str = r#"
/* WhatsApp Lite Custom CSS */
/* Hide Communities, Channels, Meta AI (Keep Chats, Status, Settings, Profile) */
div[title='Communities'], span[title='Communities'], div[aria-label='Communities'], span[aria-label='Communities'],
div[title='Channels'], span[title='Channels'], div[aria-label='Channels'], span[aria-label='Channels'],
div[title='Meta AI'], span[title='Meta AI'], div[aria-label='Meta AI'], span[aria-label='Meta AI'] {
    display: none !important;
}

/* Hide 'Get the app' banners if any */
div[data-testid='intro-title'], div[data-testid='intro-text'] {
    display: none !important;
}

/* Cool Scrollbar */
::-webkit-scrollbar {
    width: 6px !important;
    height: 6px !important;
}
::-webkit-scrollbar-track {
    background: transparent !important;
}
::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2) !important;
    border-radius: 3px !important;
}
::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3) !important;
}

div._aigw {
	max-width: 30% !important;
}
"#;

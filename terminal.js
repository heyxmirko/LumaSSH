import { Terminal } from '@xterm/xterm';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

const term = new Terminal({
    rightClickSelectsWord: true,
    scrollback: 1000,
    bracketedPaste: false,
    theme: {
        background: '#2D313C',
        foreground: '#ffffff',
        cursor: '#ffffff',
        black: '#2e3436',
        red: '#cc0000',
        green: '#4e9a06',
        yellow: '#c4a000',
        blue: '#3465a4',
        magenta: '#75507b',
        cyan: '#06989a',
        white: '#d3d7cf',
        brightBlack: '#555753',
        brightRed: '#ef2929',
        brightGreen: '#8ae234',
        brightYellow: '#fce94f',
        brightBlue: '#729fcf',
        brightMagenta: '#ad7fa8',
        brightCyan: '#34e2e2',
        brightWhite: '#eeeeec',
    }
});

// Send user input to the backend
term.onData((data) => {
    invoke('send_input', { input: data });
});

// Listen for SSH output from the backend
listen('ssh_output', (event) => {
    console.log("Received SSH output event:", event.payload); // Debugging line
    if (event.payload) {
      term.write(event.payload);
      term.scrollToBottom();
    } else {
      console.log("No payload received from SSH output event");
    }
});

term.open(document.getElementById('terminal'));

function get_connection_id_parameter() {
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    return parseInt(urlParams.get('id'))
}

async function start_ssh_session(id) {
    const [host, username, password] = await invoke('get_connection_command', {id: id});
    console.log(host, username, password)
    invoke('start_ssh_session_command', { host, username, password });
}

addEventListener("DOMContentLoaded", (event) => {
    start_ssh_session(get_connection_id_parameter())
});

// Listen for SSH errors:
listen('ssh_error', (event) => {
    const error = `SSH Error: ${event.payload}`
    console.log(error);
    term.write(error);
});
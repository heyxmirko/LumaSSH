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




// term.element.addEventListener('contextmenu', (e) => {
//     e.preventDefault();

//     const existingContextMenu = document.querySelector('.custom-context-menu');
//     if (existingContextMenu) {
//         existingContextMenu.remove();
//     }

//     // Custom context menu
//     const contextMenu = document.createElement('div');
//     contextMenu.className = 'custom-context-menu';
//     contextMenu.style.position = 'absolute';
//     contextMenu.style.top = `${e.clientY}px`;
//     contextMenu.style.left = `${e.clientX}px`;
//     contextMenu.style.background = '#fff';
//     contextMenu.style.border = '1px solid #ccc';
//     contextMenu.style.zIndex = '1000';
//     contextMenu.style.paddingTop = '5px';
//     contextMenu.style.paddingBottom = '5px';
//     contextMenu.style.paddingLeft = '20px';
//     contextMenu.style.paddingRight = '20px';
//     contextMenu.style.cursor = 'pointer';
    
//     const pasteOption = document.createElement('div');
//     pasteOption.innerText = 'Paste';
//     contextMenu.appendChild(pasteOption);

//     const copyOption = document.createElement('div');
//     copyOption.innerText = 'Copy';
//     contextMenu.appendChild(copyOption);

//     pasteOption.addEventListener('click', () => {
//         let clipboardData = null // Get cliboard data from Rust backend
//         console.log("1: ", clipboardData)
        
//         // Strip bracketed paste sequences
//         clipboardData = clipboardData.replace(/^\x1b\[200~/, '').replace(/\x1b\[201~$/, '');
        
//         console.log("2: ", clipboardData)
//         term.paste(clipboardData);

//         setTimeout(() => {
//             term.clearSelection();
//             contextMenu.remove();
//         }, 50);

//         term.focus();
//     });

//     copyOption.addEventListener('click', () => {
//         const selectedText = term.getSelection();
//         // Add selected text to clipboard using rust backend
//         term.clearSelection();
//         contextMenu.remove();
//     })

//     document.body.appendChild(contextMenu);
// });

// document.addEventListener('click', (e) => {
//     const contextMenu = document.querySelector('.custom-context-menu');
//     if (contextMenu && e.target !== contextMenu) {
//         contextMenu.remove();
//     }
// })



// window.addEventListener('resize', () => {
//     const terminalContainer = document.getElementById('terminal');
//     const terminalDragon = document.getElementById('dragon');

//     const newTerminalHeight = window.innerHeight - 40;
//     const newDragonWidth = window.innerWidth - 120;

//     console.log(newDragonWidth);

//     terminalContainer.style.height = `${newTerminalHeight}px`;
//     terminalDragon.style.width = `${newDragonWidth}px`;
// })
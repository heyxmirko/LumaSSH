import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api';

async function addConnection(name, host, username, password) {
  try {
    await invoke('add_connection_command', { name, host, username, password});
    console.log("Connection added successfully");
  } catch(error) {
    console.error('Error adding connection: ', error);
  }
}

async function loadConnections() {
  try {
    const connections = await invoke('get_connections_command');
    console.log("Connections: ", connections);
    return connections;
  } catch(error) {
    console.error("Error loading connections: ", error);
  }
}

async function deleteConnection(id) {
  try {
      await invoke('delete_connection_command', { id: parseInt(id) });
      console.log("Connection deleted successfully");
  } catch (error) {
      console.error('Error deleting connection: ', error);
  }
}


window.addEventListener("DOMContentLoaded", async () => {

  // Close button
  const closeButton = document.querySelector("#close-btn");
  closeButton.addEventListener("click", () => {
    appWindow.close();
  });


  const addConnectionForm = document.getElementById('add-connection-form');
  addConnectionForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const formData = new FormData(addConnectionForm);
    const name = formData.get('name');
    const host = formData.get('host');
    const username = formData.get('username');
    const password = formData.get('password');
    
    await addConnection(name, host, username, password);
    addConnectionForm.reset();
    await loadAndDisplayConnections();
  });

  async function loadAndDisplayConnections() {
    const connections = await loadConnections();
    const connectionsContainer = document.querySelector('.connections');
    connectionsContainer.innerHTML = '';

    connections.forEach(connection => {
      const connectionElement = createConnectionElement(connection);
      connectionsContainer.appendChild(connectionElement);
    });
  }

  function createConnectionElement(connection) {
    const [id, name, host, username] = connection;
    const connectionDiv = document.createElement('div');
    connectionDiv.className = 'connection';
    connectionDiv.oncontextmenu = (e) => showContextMenu(e, id);
    connectionDiv.onclick = () => {
    }
    connectionDiv.innerHTML = `
      <span class="connection-name">${name}</span>
              <div class="connection-details">
                <div class="user">
                  <img src="assets/icons/user.svg">
                  <span>${username}</span>
                </div>
                <div class="host">
                  <img src="assets/icons/ip.svg">
                  <span>${host}</span>
                </div>
              </div>
    `;
    return connectionDiv;
  }

  function openTerminal(host, username) {
    const command = `xterm -e ssh ${username}@${host}`;
    const { exec } = require('child_process');
    exec(command, (error) => {
        if (error) {
            console.error(`Error opening terminal: ${error.message}`);
        }
    });
}

  
  let selectedConnectionId;
  function showContextMenu(e, connectionId) {
    const contextMenu = document.getElementById("context-menu");
    contextMenu.style.display = "block";
    contextMenu.style.left = `${e.pageX}px`;
    contextMenu.style.top = `${e.pageY}px`;
    selectedConnectionId = connectionId;
    console.log(connectionId);
  }

  document.getElementById("delete-connection").addEventListener("click", async () => {
      await deleteConnection(selectedConnectionId);
      document.getElementById("context-menu").style.display = "none";
      await loadAndDisplayConnections();
  });

  window.addEventListener("click", () => {
      document.getElementById("context-menu").style.display = "none";
  });

  // Load and display connections when the app starts
  await loadAndDisplayConnections();
});



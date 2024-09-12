import { WebviewWindow } from '@tauri-apps/api/window'
const webview = new WebviewWindow('theUniqueLabel', {
  url: 'terminal.html',
})
// since the webview window is created asynchronously,
// Tauri emits the `tauri://created` and `tauri://error` to notify you of the creation response
webview.once('tauri://created', function () {
  // webview window successfully created
  console.log("new window created");
})
webview.once('tauri://error', function (e) {
    // an error occurred during webview window creation
    console.log("error creating a new window");
    console.log(e)
})

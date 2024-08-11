if (window.location.host === 'tauri.localhost') {
  document.addEventListener('contextmenu', event => event.preventDefault());
}

const { invoke } = window.__TAURI__.tauri;

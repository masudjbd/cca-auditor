import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import ErrorBoundary from './components/ErrorBoundary'
import './index.css'

// Disable right-click context menu globally (production app, not a browser)
document.addEventListener('contextmenu', (e) => e.preventDefault())

// Disable common inspect / dev tools shortcuts
document.addEventListener('keydown', (e) => {
  // F12
  if (e.key === 'F12') {
    e.preventDefault()
    return
  }
  // Cmd/Ctrl+Shift+I (Inspector), Cmd/Ctrl+Shift+J (Console),
  // Cmd/Ctrl+Shift+C (Element picker), Cmd/Opt+I (macOS Inspector)
  const meta = e.metaKey || e.ctrlKey
  if (meta && e.shiftKey && (e.key === 'I' || e.key === 'J' || e.key === 'C')) {
    e.preventDefault()
  }
  if (e.metaKey && e.altKey && (e.key === 'I' || e.key === 'i')) {
    e.preventDefault()
  }
  // Cmd/Ctrl+U (View Source)
  if (meta && (e.key === 'u' || e.key === 'U')) {
    e.preventDefault()
  }
})

// Prevent text selection drag-out on non-input elements (visual polish)
document.addEventListener('dragstart', (e) => {
  const target = e.target as HTMLElement
  if (target && target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
    e.preventDefault()
  }
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
)

import { useEffect } from 'react'
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom'
import { listen } from '@tauri-apps/api/event'
import { useAuditStream } from './hooks/useAuditStream'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Live from './pages/Live'
import Sessions from './pages/Sessions'
import ToolDetail from './pages/ToolDetail'
import Alerts from './pages/Alerts'
import Reports from './pages/Reports'
import Publish from './pages/Publish'
import Settings from './pages/Settings'
import About from './pages/About'

function AppContent() {
  const navigate = useNavigate()
  useAuditStream()

  useEffect(() => {
    const unlisten = listen<string>('navigate', (event) => {
      navigate(event.payload)
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [navigate])

  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/live" element={<Live />} />
        <Route path="/sessions" element={<Sessions />} />
        <Route path="/tools/:toolId" element={<ToolDetail />} />
        <Route path="/alerts" element={<Alerts />} />
        <Route path="/reports" element={<Reports />} />
        <Route path="/publish" element={<Publish />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/about" element={<About />} />
      </Route>
    </Routes>
  )
}

export default function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  )
}

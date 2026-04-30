import { Outlet, NavLink } from 'react-router-dom'

const navItemClass = ({ isActive }: { isActive: boolean }) =>
  `block px-4 py-2 rounded-lg transition-colors ${
    isActive
      ? 'bg-blue-50 text-blue-700 font-medium'
      : 'text-gray-700 hover:bg-gray-100'
  }`

export default function Layout() {
  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <aside className="w-64 bg-white border-r border-gray-200 shadow-sm flex flex-col">
        <div className="p-6 flex items-center gap-3 border-b border-gray-100">
          <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-indigo-900 to-slate-900 flex items-center justify-center text-lg shadow">
            🔍
          </div>
          <div>
            <h1 className="text-xl font-bold text-gray-900 leading-tight">
              CCAudit
            </h1>
            <p className="text-xs text-gray-500">AI Tool Auditor</p>
          </div>
        </div>

        <nav className="mt-4 space-y-1 px-3 flex-1">
          <NavLink to="/" end className={navItemClass}>
            <span className="inline-block w-5">📊</span> Dashboard
          </NavLink>
          <NavLink to="/live" className={navItemClass}>
            <span className="inline-block w-5">📡</span> Live Stream
          </NavLink>
          <NavLink to="/sessions" className={navItemClass}>
            <span className="inline-block w-5">📅</span> Sessions
          </NavLink>
          <NavLink to="/alerts" className={navItemClass}>
            <span className="inline-block w-5">🔔</span> Alerts
          </NavLink>
          <NavLink to="/reports" className={navItemClass}>
            <span className="inline-block w-5">📄</span> Reports
          </NavLink>
          <NavLink to="/publish" className={navItemClass}>
            <span className="inline-block w-5">🛡️</span> Publish
          </NavLink>

          <div className="border-t border-gray-100 my-3" />

          <NavLink to="/settings" className={navItemClass}>
            <span className="inline-block w-5">⚙️</span> Settings
          </NavLink>
          <NavLink to="/about" className={navItemClass}>
            <span className="inline-block w-5">ℹ️</span> About
          </NavLink>
        </nav>

        <div className="p-3 text-xs text-gray-400 border-t border-gray-100">
          v0.1.0 · MIT
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  )
}

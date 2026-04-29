import { Outlet, Link } from 'react-router-dom'

export default function Layout() {
  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <aside className="w-64 bg-white border-r border-gray-200 shadow-sm">
        <div className="p-6">
          <h1 className="text-2xl font-bold text-gray-900">CCAudit</h1>
          <p className="text-sm text-gray-500 mt-1">AI Tool Auditor</p>
        </div>

        <nav className="mt-8 space-y-1 px-4">
          <Link
            to="/"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100 font-medium"
          >
            Dashboard
          </Link>
          <Link
            to="/live"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100"
          >
            Live Stream
          </Link>
          <Link
            to="/sessions"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100"
          >
            Sessions
          </Link>
          <Link
            to="/alerts"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100"
          >
            Alerts
          </Link>
          <Link
            to="/reports"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100"
          >
            Reports
          </Link>
          <Link
            to="/publish"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100"
          >
            Publish
          </Link>
          <Link
            to="/settings"
            className="block px-4 py-2 rounded-lg text-gray-700 hover:bg-gray-100"
          >
            Settings
          </Link>
        </nav>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  )
}

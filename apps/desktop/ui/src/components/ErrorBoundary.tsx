import { Component, ErrorInfo, ReactNode } from 'react'

interface Props {
  children: ReactNode
}

interface State {
  hasError: boolean
  error?: Error
  errorInfo?: ErrorInfo
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('UI crash:', error, errorInfo)
    this.setState({ errorInfo })
  }

  reset = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined })
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-gray-50 flex items-center justify-center p-8">
          <div className="bg-white rounded-lg shadow-lg max-w-2xl w-full p-8">
            <div className="flex items-start gap-4">
              <span className="text-4xl">💥</span>
              <div className="flex-1">
                <h1 className="text-2xl font-bold text-gray-900">
                  Something went wrong
                </h1>
                <p className="text-gray-600 mt-2">
                  CCAudit's UI crashed. The backend monitors are still running and
                  collecting data — only the window encountered an error.
                </p>
                {this.state.error && (
                  <div className="mt-4 p-4 bg-red-50 border border-red-200 rounded">
                    <p className="text-sm font-mono text-red-900 break-all">
                      {this.state.error.message}
                    </p>
                  </div>
                )}
                {this.state.errorInfo && (
                  <details className="mt-3">
                    <summary className="text-sm text-gray-500 cursor-pointer">
                      Stack trace
                    </summary>
                    <pre className="mt-2 p-3 bg-gray-50 rounded text-xs text-gray-700 overflow-auto max-h-64">
                      {this.state.errorInfo.componentStack}
                    </pre>
                  </details>
                )}
                <div className="mt-6 flex gap-3">
                  <button
                    onClick={this.reset}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700"
                  >
                    Try Again
                  </button>
                  <button
                    onClick={() => window.location.reload()}
                    className="px-4 py-2 border border-gray-300 rounded-lg font-medium hover:bg-gray-50"
                  >
                    Reload App
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )
    }
    return this.props.children
  }
}

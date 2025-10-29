import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './dev/worker-detector' // dev-only helper (no side effects in prod)
import App from './App.tsx'

// Cross-origin isolation diagnostic
if (typeof crossOriginIsolated !== 'undefined') {
  console.log('[Isolation]', crossOriginIsolated ? 'ON (MT available)' : 'OFF (fallback ST)');
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)

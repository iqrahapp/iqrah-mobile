// Development-only worker detector to catch problematic worker URLs
if (import.meta.env.DEV) {
  const OrigWorker = window.Worker;
  // @ts-expect-error override for diagnostics
  window.Worker = function(url: string | URL, opts?: WorkerOptions) {
    const u = typeof url === 'string' ? url : url?.toString();
    if (u && u.includes('worker') && u.includes('worker_file')) {
      // eslint-disable-next-line no-console
      console.warn('[Worker-Detect] Problematic worker URL detected:', u, '\n', new Error().stack);
    }
    // @ts-ignore
    return new OrigWorker(url, opts);
  };
  console.log('[Worker-Detect] Worker detector installed');
}

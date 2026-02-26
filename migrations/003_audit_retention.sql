-- COM-01: Audit log retention. Run this migration once.
-- Retention policy: 90 days. Schedule a cron or app job to run the DELETE below.

-- Optional: Add a comment to the table for documentation.
COMMENT ON TABLE audit_logs IS 'Metadata only; no PII. Retention: 90 days. Run: DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL ''90 days'';';

-- One-time cleanup (optional; prefer scheduled job):
-- DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '90 days';

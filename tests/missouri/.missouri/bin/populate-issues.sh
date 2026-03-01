#!/bin/bash
set -e

# Create projects
tisket project create core
tisket project create docs
tisket project create infra

# === CORE PROJECT (80 issues) ===

# Marker issues
tisket issue create 'Fix OAuth2 token refresh bug' -p core --priority 1 --assignee alice --labels 'bug,auth,security'
tisket issue create 'Implement GraphQL subscription handler' -p core --priority 2 --assignee bob --labels 'feature,api'
tisket issue create 'Add search functionality to dashboard' -p core --priority 3 --assignee alice --labels 'feature,search,ui'
tisket issue create 'CRITICAL production memory leak in worker pool' -p core --priority 1 --labels 'bug,urgent,performance'
tisket issue create 'Refactor PostgreSQL connection pooling' -p core --priority 2 --assignee dave --labels 'refactor,database,performance'
tisket issue create 'Implement WebSocket heartbeat mechanism' -p core --priority 2 --assignee carol --labels 'feature,api'
tisket issue create 'Fix race condition in queue processor' -p core --priority 1 --assignee alice --labels 'bug,performance' --status in_progress

# Core issues (open)
tisket issue create 'Add rate limiting to authentication endpoints' -p core --priority 2 --assignee frank --labels 'security,auth'
tisket issue create 'Implement JWT refresh token rotation' -p core --priority 2 --assignee carol --labels 'auth,security'
tisket issue create 'Add two-factor authentication support' -p core --priority 2 --assignee alice --labels 'auth,feature'
tisket issue create 'Fix session invalidation on password change' -p core --priority 1 --assignee bob --labels 'bug,auth'
tisket issue create 'Migrate payment gateway to Stripe v3 API' -p core --priority 2 --assignee dave --labels 'migration,api' --status in_progress
tisket issue create 'Add webhook signature validation for payment events' -p core --priority 1 --assignee eve --labels 'security,api'
tisket issue create 'Fix duplicate charge on payment retry' -p core --priority 1 --labels 'bug,urgent'
tisket issue create 'Implement idempotency keys for payment requests' -p core --priority 2 --assignee frank --labels 'feature,api'
tisket issue create 'Add email notification for failed payments' -p core --priority 3 --assignee carol --labels 'feature'
tisket issue create 'Implement push notification delivery tracking' -p core --priority 3 --assignee bob --labels 'feature'
tisket issue create 'Fix notification queue backpressure handling' -p core --priority 2 --assignee dave --labels 'bug,performance'
tisket issue create 'Add notification preference management API' -p core --priority 3 --labels 'feature,api'
tisket issue create 'Implement Redis cache invalidation strategy' -p core --priority 2 --assignee alice --labels 'performance,database'
tisket issue create 'Add cache warming on application startup' -p core --priority 3 --assignee frank --labels 'performance'
tisket issue create 'Fix stale cache entries in user profile service' -p core --priority 2 --assignee carol --labels 'bug,performance'
tisket issue create 'Implement distributed cache locking' -p core --priority 2 --assignee dave --labels 'feature,performance'
tisket issue create 'Add structured logging with trace IDs' -p core --priority 3 --assignee bob --labels 'feature,devops'
tisket issue create 'Fix log rotation causing disk space exhaustion' -p core --priority 1 --labels 'bug,urgent,devops'
tisket issue create 'Migrate logging pipeline to OpenTelemetry' -p core --priority 2 --assignee eve --labels 'migration,devops'
tisket issue create 'Add audit logging for admin actions' -p core --priority 2 --assignee alice --labels 'security,feature'
tisket issue create 'Implement full-text index on messages table' -p core --priority 2 --assignee dave --labels 'database,performance'
tisket issue create 'Fix N+1 query in user activity feed' -p core --priority 2 --assignee bob --labels 'bug,database,performance'
tisket issue create 'Add database query timeout enforcement' -p core --priority 2 --labels 'database,performance'
tisket issue create 'Optimize slow report generation query' -p core --priority 3 --assignee carol --labels 'performance,database'
tisket issue create 'Add pagination to admin user listing endpoint' -p core --priority 3 --assignee frank --labels 'api,feature'
tisket issue create 'Fix CORS preflight handling for mobile clients' -p core --priority 2 --assignee eve --labels 'bug,api'
tisket issue create 'Implement API versioning strategy' -p core --priority 2 --assignee alice --labels 'api,refactor'
tisket issue create 'Add request body size limits to upload endpoints' -p core --priority 2 --assignee bob --labels 'security,api'
tisket issue create 'Fix error response format inconsistency across endpoints' -p core --priority 3 --labels 'bug,api'
tisket issue create 'Add input sanitization to user profile fields' -p core --priority 2 --assignee carol --labels 'security,feature'
tisket issue create 'Implement form validation error messaging' -p core --priority 3 --assignee frank --labels 'ui,feature'
tisket issue create 'Fix dark mode color contrast on tooltip components' -p core --priority 3 --assignee dave --labels 'bug,ui'
tisket issue create 'Add accessibility labels to icon-only buttons' -p core --priority 3 --labels 'ui,feature'
tisket issue create 'Implement infinite scroll for activity timeline' -p core --priority 3 --assignee eve --labels 'ui,feature'
tisket issue create 'Fix modal focus trap on keyboard navigation' -p core --priority 2 --assignee alice --labels 'bug,ui'
tisket issue create 'Add loading skeleton for dashboard widgets' -p core --priority 4 --labels 'ui,enhancement'
tisket issue create 'Refactor authentication middleware into composable hooks' -p core --priority 3 --assignee bob --labels 'refactor,auth'
tisket issue create 'Extract payment processing into dedicated service' -p core --priority 2 --assignee carol --labels 'refactor,feature'
tisket issue create 'Consolidate duplicate user validation logic' -p core --priority 3 --assignee dave --labels 'refactor'
tisket issue create 'Refactor error handling to use typed result wrappers' -p core --priority 3 --labels 'refactor'
tisket issue create 'Add integration tests for payment flow' -p core --priority 2 --assignee eve --labels 'testing'
tisket issue create 'Increase unit test coverage for auth module' -p core --priority 3 --assignee frank --labels 'testing,auth'
tisket issue create 'Add end-to-end tests for user onboarding' -p core --priority 3 --assignee alice --labels 'testing,feature'
tisket issue create 'Fix flaky test in notification service suite' -p core --priority 2 --assignee bob --labels 'bug,testing'
tisket issue create 'Write API contract tests for partner integrations' -p core --priority 2 --labels 'testing,api,docs'
tisket issue create 'Update API reference docs for v2 endpoints' -p core --priority 3 --assignee carol --labels 'docs,api'
tisket issue create 'Add runbook for database failover procedure' -p core --priority 2 --assignee dave --labels 'docs,devops'
tisket issue create 'Document rate limiting behavior for integrators' -p core --priority 3 --labels 'docs,api'
tisket issue create 'Implement blue-green deployment pipeline' -p core --priority 2 --assignee eve --labels 'devops,feature'
tisket issue create 'Fix Kubernetes pod autoscaler thrashing under load' -p core --priority 1 --assignee frank --labels 'bug,devops,performance'
tisket issue create 'Add health check endpoint for load balancer probes' -p core --priority 2 --assignee alice --labels 'devops,feature'
tisket issue create 'Automate database backup verification' -p core --priority 2 --labels 'devops,database'

# Core issues (to be closed)
tisket issue create 'Resolve deprecated bcrypt hashing algorithm' -p core --priority 2 --assignee eve --labels 'security,migration'
tisket issue create 'Upgrade dependency versions to resolve known vulnerabilities' -p core --priority 1 --assignee bob --labels 'security,urgent' --status todo
tisket issue create 'Remove hardcoded API keys from configuration files' -p core --priority 1 --assignee carol --labels 'security,urgent' --status todo
tisket issue create 'Fix XSS vulnerability in comment rendering' -p core --priority 1 --assignee dave --labels 'bug,security,urgent'
tisket issue create 'Patch SQL injection vector in report filter' -p core --priority 1 --assignee eve --labels 'bug,security,urgent'
tisket issue create 'Implement Content Security Policy headers' -p core --priority 2 --assignee frank --labels 'security,feature'
tisket issue create 'Add HSTS preload to production domains' -p core --priority 2 --labels 'security,devops'
tisket issue create 'Migrate user avatars from local disk to object storage' -p core --priority 3 --assignee alice --labels 'migration,devops'
tisket issue create 'Complete transition from REST to typed RPC layer' -p core --priority 2 --assignee bob --labels 'migration,api'
tisket issue create 'Remove legacy feature flag evaluation code' -p core --priority 3 --labels 'refactor'
tisket issue create 'Drop unused columns from accounts table' -p core --priority 3 --assignee carol --labels 'database,migration'
tisket issue create 'Archive historical event log partitions' -p core --priority 4 --assignee dave --labels 'database,devops'
tisket issue create 'Fix broken links in developer documentation' -p core --priority 4 --assignee eve --labels 'bug,docs'
tisket issue create 'Update onboarding guide for new team members' -p core --priority 4 --labels 'docs'
tisket issue create 'Clean up orphaned test fixtures in database' -p core --priority 3 --assignee frank --labels 'testing,database'
tisket issue create 'Decommission shadow-mode legacy authentication service' -p core --priority 2 --assignee alice --labels 'devops,auth'
tisket issue create 'Complete password hashing algorithm upgrade rollout' -p core --priority 2 --assignee bob --labels 'security,migration'
tisket issue create 'Mark beta notification channels as stable' -p core --priority 3 --labels 'feature'
tisket issue create 'Close out deprecated v1 API endpoints' -p core --priority 2 --assignee carol --labels 'api,migration'
tisket issue create 'Verify production TLS certificate renewal automation' -p core --priority 2 --assignee dave --labels 'devops,security'
tisket issue create 'Finalize GDPR data retention policy implementation' -p core --priority 1 --assignee eve --labels 'security,feature'

# === DOCS PROJECT (40 issues) ===

# Marker issues
tisket issue create 'Write API authentication guide' -p docs --assignee carol --labels 'docs,auth,guide'
tisket issue create 'Document PostgreSQL backup procedures' -p docs --priority 2 --assignee dave --labels 'docs,database'

# Docs issues (open)
tisket issue create 'Add onboarding guide for new developers' -p docs --assignee alice --labels 'docs,guide'
tisket issue create 'Update REST API endpoint reference' -p docs --priority 3 --labels 'docs,api'
tisket issue create 'Write troubleshooting guide for deployment failures' -p docs --labels 'docs,guide'
tisket issue create 'Create user guide for the admin dashboard' -p docs --assignee bob --priority 3 --labels 'docs,tutorial'
tisket issue create 'Document database migration runbook' -p docs --priority 2 --labels 'docs,migration'
tisket issue create 'Improve README for the core library' -p docs --assignee frank --labels 'docs'
tisket issue create 'Write architecture overview for the backend services' -p docs --priority 3 --assignee alice --labels 'docs'
tisket issue create 'Add changelog entry for v2.4 release' -p docs --labels 'docs'
tisket issue create 'Document environment variable reference' -p docs --assignee eve --labels 'docs,api'
tisket issue create 'Write tutorial for integrating the payments module' -p docs --priority 3 --labels 'docs,tutorial'
tisket issue create 'Create rate limiting configuration guide' -p docs --assignee carol --labels 'docs,api'
tisket issue create 'Document error codes returned by the API' -p docs --priority 3 --labels 'docs,api'
tisket issue create 'Write step-by-step data import tutorial' -p docs --assignee bob --labels 'docs,tutorial'
tisket issue create 'Add troubleshooting section to the CLI reference' -p docs --labels 'docs'
tisket issue create 'Document token refresh flow for mobile clients' -p docs --assignee dave --labels 'docs,auth'
tisket issue create 'Update migration guide for v3 schema changes' -p docs --priority 2 --labels 'docs,migration'
tisket issue create 'Write testing strategy guide for API consumers' -p docs --assignee frank --labels 'docs,testing,guide'
tisket issue create 'Document webhook event payload formats' -p docs --priority 3 --assignee alice --labels 'docs,api'
tisket issue create 'Create contributor guide for open source repo' -p docs --labels 'docs,guide'
tisket issue create 'Write glossary of domain terms' -p docs --assignee eve --labels 'docs'
tisket issue create 'Document service-level objectives and definitions' -p docs --priority 4 --labels 'docs'
tisket issue create 'Add examples to the SDK quickstart guide' -p docs --assignee carol --labels 'docs,tutorial'
tisket issue create 'Write incident response runbook' -p docs --priority 2 --assignee alice --labels 'docs'
tisket issue create 'Document CI pipeline configuration options' -p docs --labels 'docs,guide'
tisket issue create 'Create end-to-end tutorial for the reporting feature' -p docs --assignee bob --labels 'docs,tutorial'
tisket issue create 'Write session management reference guide' -p docs --labels 'docs,auth'
tisket issue create 'Document feature flag system usage' -p docs --assignee frank --labels 'docs,guide'
tisket issue create 'Add deprecation notices to legacy API docs' -p docs --priority 2 --labels 'docs,api'

# Docs issues (to be closed)
tisket issue create 'Document caching layer behavior and TTL settings' -p docs --assignee dave --labels 'docs,api'
tisket issue create 'Write legacy client upgrade guide' -p docs --priority 3 --labels 'docs,migration'
tisket issue create 'Document initial setup for local development environment' -p docs --assignee eve --labels 'docs,tutorial'
tisket issue create 'Add code samples to the authorization reference' -p docs --labels 'docs,auth'
tisket issue create 'Write data retention policy documentation' -p docs --priority 4 --assignee carol --labels 'docs'
tisket issue create 'Document container orchestration setup guide' -p docs --labels 'docs,guide'
tisket issue create 'Update README badges and build status links' -p docs --assignee bob --labels 'docs'
tisket issue create 'Write guide for configuring multi-region deployments' -p docs --priority 3 --labels 'docs,guide'
tisket issue create 'Document log aggregation and monitoring setup' -p docs --assignee frank --labels 'docs'
tisket issue create 'Write API versioning policy document' -p docs --labels 'docs,api'

# === INFRA PROJECT (30 issues) ===

# Marker issues
tisket issue create 'Migrate Kubernetes cluster to version 1.29' -p infra --priority 1 --assignee dave --labels 'devops,migration'
tisket issue create 'Implement Prometheus alerting rules for memory' -p infra --priority 2 --assignee frank --labels 'monitoring,devops'

# Infra issues (open)
tisket issue create 'Set up container registry with image scanning' -p infra --priority 2 --labels 'devops,security'
tisket issue create 'Configure load balancer health check thresholds' -p infra --assignee alice --labels 'devops'
tisket issue create 'Automate SSL certificate renewal with certbot' -p infra --priority 3 --labels 'security,devops'
tisket issue create 'Implement offsite backup replication for block storage' -p infra --assignee bob --labels 'devops'
tisket issue create 'Build CI pipeline for infrastructure-as-code validation' -p infra --priority 2 --assignee carol --labels 'devops,testing'
tisket issue create 'Create monitoring dashboard for API latency metrics' -p infra --labels 'monitoring'
tisket issue create 'Aggregate application logs into centralized sink' -p infra --priority 3 --assignee eve --labels 'monitoring,devops'
tisket issue create 'Define network policies for pod-to-pod isolation' -p infra --priority 2 --labels 'security,devops'
tisket issue create 'Configure horizontal auto-scaling for worker nodes' -p infra --assignee frank --labels 'performance,devops'
tisket issue create 'Write disaster recovery runbook for zone failover' -p infra --priority 1 --assignee dave --labels 'devops'
tisket issue create 'Rotate IAM service account credentials' -p infra --priority 2 --labels 'security'
tisket issue create 'Tune database connection pool limits under load' -p infra --assignee alice --labels 'database,performance'
tisket issue create 'Add CD stage for canary deployments to staging' -p infra --priority 3 --assignee bob --labels 'devops'
tisket issue create 'Benchmark storage IOPS for high-throughput workloads' -p infra --labels 'performance,devops'
tisket issue create 'Harden container base images to remove unused packages' -p infra --priority 3 --assignee carol --labels 'security,devops'
tisket issue create 'Set up VPN gateway for remote developer access' -p infra --priority 2 --labels 'security,devops'
tisket issue create 'Configure log retention and archival policy' -p infra --assignee eve --labels 'devops'
tisket issue create 'Implement alerting for disk utilization thresholds' -p infra --priority 3 --labels 'monitoring'

# Infra issues (to be closed)
tisket issue create 'Patch kernel vulnerability on bare-metal hosts' -p infra --priority 1 --assignee frank --labels 'security'
tisket issue create 'Decommission legacy load balancer in us-east-1' -p infra --labels 'devops'
tisket issue create 'Remove stale DNS records from production zone' -p infra --assignee alice --labels 'devops'
tisket issue create 'Upgrade Terraform provider versions to unblock CI' -p infra --priority 3 --labels 'devops,migration'
tisket issue create 'Replace self-signed cert on internal dashboard' -p infra --assignee bob --labels 'security'
tisket issue create 'Archive unused container images older than 90 days' -p infra --labels 'devops'
tisket issue create 'Fix broken health check endpoint on staging cluster' -p infra --priority 2 --assignee carol --labels 'bug,devops'
tisket issue create 'Update firewall rules to block deprecated TLS versions' -p infra --priority 2 --labels 'security'
tisket issue create 'Migrate artifact storage bucket to new region' -p infra --assignee dave --labels 'devops,migration'
tisket issue create 'Resolve flaky integration tests in deployment pipeline' -p infra --priority 3 --labels 'testing,devops'

# === CLOSE ISSUES ===

# Core closes
tisket issue close resolve-deprecated-bcrypt-hashing-algorithm
tisket issue close upgrade-dependency-versions-to-resolve-known-vulnerabilities
tisket issue close remove-hardcoded-api-keys-from-configuration-files
tisket issue close fix-xss-vulnerability-in-comment-rendering
tisket issue close patch-sql-injection-vector-in-report-filter
tisket issue close implement-content-security-policy-headers
tisket issue close add-hsts-preload-to-production-domains
tisket issue close migrate-user-avatars-from-local-disk-to-object-storage
tisket issue close complete-transition-from-rest-to-typed-rpc-layer
tisket issue close remove-legacy-feature-flag-evaluation-code
tisket issue close drop-unused-columns-from-accounts-table
tisket issue close archive-historical-event-log-partitions
tisket issue close fix-broken-links-in-developer-documentation
tisket issue close update-onboarding-guide-for-new-team-members
tisket issue close clean-up-orphaned-test-fixtures-in-database
tisket issue close decommission-shadow-mode-legacy-authentication-service
tisket issue close complete-password-hashing-algorithm-upgrade-rollout
tisket issue close mark-beta-notification-channels-as-stable
tisket issue close close-out-deprecated-v1-api-endpoints
tisket issue close verify-production-tls-certificate-renewal-automation
tisket issue close finalize-gdpr-data-retention-policy-implementation

# Docs closes
tisket issue close document-caching-layer-behavior-and-ttl-settings
tisket issue close write-legacy-client-upgrade-guide
tisket issue close document-initial-setup-for-local-development-environment
tisket issue close add-code-samples-to-the-authorization-reference
tisket issue close write-data-retention-policy-documentation
tisket issue close document-container-orchestration-setup-guide
tisket issue close update-readme-badges-and-build-status-links
tisket issue close write-guide-for-configuring-multi-region-deployments
tisket issue close document-log-aggregation-and-monitoring-setup
tisket issue close write-api-versioning-policy-document

# Infra closes
tisket issue close patch-kernel-vulnerability-on-bare-metal-hosts
tisket issue close decommission-legacy-load-balancer-in-us-east-1
tisket issue close remove-stale-dns-records-from-production-zone
tisket issue close upgrade-terraform-provider-versions-to-unblock-ci
tisket issue close replace-self-signed-cert-on-internal-dashboard
tisket issue close archive-unused-container-images-older-than-90-days
tisket issue close fix-broken-health-check-endpoint-on-staging-cluster
tisket issue close update-firewall-rules-to-block-deprecated-tls-versions
tisket issue close migrate-artifact-storage-bucket-to-new-region
tisket issue close resolve-flaky-integration-tests-in-deployment-pipeline

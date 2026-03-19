# Alertmanager Webhook v4 Payload Format

## Payload Schema

```json
{
  "version": "4",
  "groupKey": "<string>",
  "truncatedAlerts": 0,
  "status": "<resolved|firing>",
  "receiver": "<string>",
  "groupLabels": { "<labelname>": "<labelvalue>" },
  "commonLabels": { "<labelname>": "<labelvalue>" },
  "commonAnnotations": { "<annotationname>": "<value>" },
  "externalURL": "<string>",
  "alerts": [
    {
      "status": "<resolved|firing>",
      "labels": { "<labelname>": "<labelvalue>" },
      "annotations": { "<annotationname>": "<value>" },
      "startsAt": "<rfc3339>",
      "endsAt": "<rfc3339>",
      "generatorURL": "<string>",
      "fingerprint": "<string>"
    }
  ]
}
```

## Key Details
- version is always "4" (hardcoded in Alertmanager)
- alerts[] can contain multiple alerts (batching)
- endsAt is "0001-01-01T00:00:00Z" when alert is still firing
- fingerprint is unique per alert instance
- Alertmanager retries on 5xx, does NOT retry on 4xx
- Content-Type is application/json, method is always POST

## Auth Options for Webhook Targets
- Basic auth: http_config.basic_auth
- Bearer token: http_config.authorization.type + credentials
- OAuth 2.0: http_config.oauth2
- mTLS: http_config.tls_config
- Custom headers: http_config.http_headers

## DDoS Label Mapping Convention
- labels.vector → AttackEvent.vector
- labels.victim_ip or labels.instance (strip port) → victim_ip
- annotations.bps → bps (parse as i64)
- annotations.pps → pps (parse as i64)
- labels.severity → confidence (critical=0.9, warning=0.7, info=0.5)
- fingerprint → external_event_id

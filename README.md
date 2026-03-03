# Herodot

Server application for persisting sensor data.
It works in combination with [Percepter](https://github.com/ladekjaer/percepter).

It is written and used for educational purposes.

## Terminology

A *reading* represents the output of a sensing instrument.
A *record* is a single reading annotated with the timestamp of its capture and
a unique identifier. Therefore, a record always has three fields: `id`,
`timestamp`, and `reading`. A reading is generic. It has its own type for each
type of sensor. Records and readings are externally defined in
[rerec](https://github.com/ladekjaer/rerec), which can be installed with cargo.
See the [rerec documentation](https://docs.rs/rerec) for more information.

## Examples

Add a new record:

```bash
curl -v http://localhost:8080/api/records -H 'access_key: 99ea32d6-e0dc-4b2c-9802-6eaeaf55bbac' -H 'Content-Type: application/json' -X PUT -d '{"id": "7e9b1a33-05fb-48e3-86b6-21ddc873c06f", "timestamp": "2026-02-27T09:32:45+00:00", "reading": {"DS18B20": {"device_name": "0000003e33d5", "raw_reading": 22123}}}'
```

# x509 test data

The three `*.key.pem` files (`leaf`, `localhost_leaf`, `rsa_leaf`) are throwaway test keys.
`gencerts.py` in this directory writes them, with the certificates, for the certificate
verifier tests (ADR 0368). They protect nothing.

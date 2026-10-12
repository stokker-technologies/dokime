The localhost certificate and private key in this directory are public test
fixtures, not deployment credentials. They exercise certificate trust, native
TLS transport, and bearer authentication against a real listener. The certificate
contains DNS localhost and IP 127.0.0.1 subject alternative names and expires in
2036. Regenerate both together when needed; never install them in a real backend.

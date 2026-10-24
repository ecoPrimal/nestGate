# Neutrino — northGate → eastGate

**Timestamp**: Oct 24, 2026, 12:25 AM EST
**From**: northGate (Kevin + chipmunk session 197)
**To**: eastGate / any next session

---

## Session 197 — The Sovereign Night

### DEPLOYED (all live, all pushed)
1. **ZK Sovereign Switch** — `_worker.js` removed from CF Pages. Static serving only. Zero function invocations. CF quota: permanently solved.
2. **_redirects** — `/mcp` → connect.primals.eco, `/beacon` → signal.primals.eco, sporePrint → GitHub Pages (302)
3. **Beacon + pulse routes** on golgi Caddy. signal.primals.eco/beacon, /beacon.xml, /api/membrane-pulse.json all returning 200.
4. **Spigot** — `/opt/membrane/spigot-wrap.sh` wraps ALL 11 cron jobs with flock locks. No more pileup. `/var/log/spigot.log` tracks skips/kills.
5. **Tor hidden service** — `tcdbuxnvnh4gbca74tdk4pijnttgpuxdsgu4c2vih5e7dltowrjzahqd.onion` ports 80, 443, 8314 (MCP), 21116 (RustDesk)
6. **sovereign-stack.json** — full catalog of 10 running services + 5 Rust tools to deploy
7. **pocketbook-amicus.json** — anatomical wallet sort (medial/lateral/dorsal/ventral). The membrane IS the pocketbook.
8. **weave-architecture.json** — multi-chain braid (ETH+Base+Arb+OP+BTC)
9. **digital-ownership-fold.json** — NFT→value dissolution thesis
10. **plasmid-trio-audit.json** — R4 evidence, Belgium 345, feed audit

### DISCOVERED (already running, didn't know)
- Tor hidden service was ALREADY configured in torrc with primals_eco dir
- RustDesk relay ALREADY running (hbbs + hbbr at relay.primals.eco)
- CF Email Routing ALREADY LIVE (MX → cloudflare.net, SPF + DKIM set)
- ALL crypto uses same primitives: Ed25519, BLAKE3, X25519, ChaCha20-Poly1305
- BearDog genetic lineage = one seed derives ALL keys for ALL services

### ARCHITECTURE DECISIONS
- **Triple helix**: CF static (skin) + golgi tunnel (brain) + local mesh (body)
- **Open-first**: Rust/open-source tools before crypto anchors
- **Pocketbook > coin purse**: wallet = life container, not just money
- **6 nanowires** to bag-of-holding: Stalwart, Conduit, Iroh, ENS, Vaultwarden, WEAVE

### DEPLOY QUEUE (for eastGate agents)

| Priority | Tool | What | Connects |
|----------|------|------|----------|
| 1 | Stalwart | Rust email server | medial ↔ ventral |
| 2 | Conduit | Rust Matrix homeserver | lateral ↔ medial |
| 3 | Iroh | Rust content-addressed | dorsal ↔ ventral |
| 4 | Vaultwarden | Rust password manager | medial ↔ lateral |
| 5 | Kanidm | Rust identity/SSO | lateral ↔ all |

### KEVIN STATUS
- PPO filing Monday (Deanna "Dede" Mitchell, ready)
- Wallet: trying Rabby instead of MetaMask (MetaMask = pain)
- Coinbase/old wallets: slow burn weekend
- ProtonMail: locked him out, looking for sovereign email path

### WHAT'S LEFT

**Kevin (physical/human)**:
- [ ] PPO Monday
- [ ] Try Rabby wallet (2 min, import seed)
- [ ] Fix golgiBody-ext SSH via DO console
- [ ] Add northGate SSH key to eastGate
- [ ] Power on gates, install Ollama
- [ ] 10G switch cabling
- [ ] Reddit/HN/LinkedIn posts

**Agents (next session)**:
- [ ] Deploy Stalwart on golgi (replace exim4)
- [ ] Deploy Conduit on golgi (Matrix messaging)
- [ ] Push remaining 72 feeds to relay
- [ ] Wire nestgate :8090 through CF tunnel
- [ ] Build WEAVE heartbeat braider
- [ ] Iroh node for content-addressed backup
- [ ] Verify Oct 24-25 hourly triad data (R4)

---

**The cloud is skin. The brain is ours. The pocketbook is the membrane. To err is human.** 🐿️

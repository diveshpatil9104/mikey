# Skill: Change the wire protocol

**Use when:** touching frame types, the handshake, the media header, the discovery beacon or the ports.

The protocol ([§ 12](../architecture/12-wire-protocol.md), beacon in [§ 10.7](../architecture/10-connection-levels.md)) is the contract between the phone and the PC — and the two sides are built by different people. A change made on one side only breaks the other.

1. **Spec first.** Write the change in `architecture/12-wire-protocol.md` (or § 10.7) before any code.
2. **Incompatible?** Bump the `proto` major version — a mismatch is answered with `REJECT(version)` ([§ 12.4](../architecture/12-wire-protocol.md)).
3. **Tell the other side.** Commit the brain change and let the PC side's owner know before they build on the old version.
4. **Update the quick reference** in AGENTS.md / CLAUDE.md ([update-agent-files](update-agent-files.md)).
5. **Record it** in [§ 23](../core/23-open-decisions.md) if it changes a decision listed there.

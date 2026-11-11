# Compliance

## CRA Applicability Worksheet

Assessment dated 8 November 2026. These are working conclusions for owner and legal
review, not a declaration of conformity. Reassess when distribution, monetisation,
plugins or intended functions change.

| Question | Assessment | Basis |
| --- | --- | --- |
| Product with digital elements? | Yes: downloadable test software with device and network connections. | CRA Articles 2–3 |
| Commercial activity? | Commercial intent warrants planning for scope. Confirm actual EU market placement and monetisation; Apache-2.0 alone does not establish exemption. | Articles 2–3; recitals 18–19 |
| Manufacturer? | Stokker Technologies is the prospective manufacturer when supplying Dokime under its name commercially. Confirm the responsible legal entity. Steward status is not an elective exemption. | Articles 3, 13, 24 |
| Class? | Provisionally ordinary software: the described test hub is not identified as an Annex III important or Annex IV critical product. Review actual security functions and bundled components. | Articles 7–8; Annexes III–IV |
| Conformity route? | Provisionally internal control, Module A. Complete risk assessment and technical documentation before issuing the EU declaration and CE marking. No conformity claim today. | Articles 28–32; Annexes I, VII–VIII |

Sources: [CRA, Regulation (EU) 2024/2847](https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng)
and [Commission Open-Source Guidance](https://digital-strategy.ec.europa.eu/en/policies/cra-open-source).

## Obligations and Dates

Article 14 reporting has applied since **11 September 2026**: for actively exploited
vulnerabilities and severe security incidents, early warning within **24 hours** and
notification within **72 hours** of awareness to the coordinating CSIRT and ENISA
through the single reporting platform. Vulnerability final reports follow within
14 days after a corrective or mitigating measure becomes available; incident final
reports within one month after notification. Business-day support targets do not
extend these clocks. See [CRA Articles 14–16 and 71](https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng).

Main obligations apply **11 December 2027**; conformity-assessment-body provisions
began **11 June 2026**. Support generally lasts at least five years unless expected
use is shorter. Technical documentation must be retained for ten years or the
support period, whichever is longer. The proposed latest-stable maintenance policy
requires reconciliation with these obligations, including security-update availability.
See [CRA Articles 13 and 71](https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng).

Risk assessment, secure development, vulnerability handling, component inventory,
instructions and conformity evidence remain manufacturer responsibilities; an SBOM
or attestation alone does not demonstrate compliance. See the
[Commission Manufacturer Guidance](https://digital-strategy.ec.europa.eu/en/policies/cra-manufacturers).

## Evidence and Remaining Work

Existing evidence includes [build instructions](../CONTRIBUTING.md), release tooling,
checksums and Windows/Linux CI. [SECURITY.md](../SECURITY.md) is pending publication.
Release attestations and SBOMs are pending supply-chain delivery and verification on
published assets; do not infer their presence on older releases. Executed per-layer
test counts in release notes remain pending release-tooling implementation.

Owners below are responsible roles to be assigned by the product owner. Dates are
proposed targets, not evidence of completion or extensions of legal deadlines.

| Remaining Work | Owner | Target |
| --- | --- | --- |
| Confirm scope, legal entity, market history and classification | Product owner and legal adviser | 20 November 2026 |
| Assign reporting lead and backup; verify platform access, escalation and 24/72-hour readiness | Security owner | Immediate; readiness review 13 November 2026 |
| Publish private disclosure channel and rehearse incident handling | Security owner | 13 November 2026 |
| Verify release SBOM, attestations, advisory triage and dependency ownership | Supply-chain owner | 30 November 2026 |
| Publish exact-tag CI URL and executed per-layer counts, including failures and missing proof | Release owner | Next stable release |
| Confirm support contacts, staffing, EOL notice and funded CRA support period | Product owner | 15 December 2026 |
| Complete risk assessment, Annex I gap analysis and remediation plan | Engineering and security owners | 31 March 2027 |
| Complete technical file, instructions, support-period rationale, conformity assessment and declaration/CE requirements | Product owner and legal adviser | Before applicable market placement; readiness by 30 November 2027 |

## Customer Responsibilities

Customers determine their NIS2 entity classification and applicable national duties.
Dokime evidence may support supplier review; it does not establish customer compliance.
Review supply-chain security and incident procedures under
[NIS2, Directive (EU) 2022/2555, Articles 2–3, 21 and 23](https://eur-lex.europa.eu/eli/dir/2022/2555/oj/eng).

Customers classify their equipment, plugins, test data, technology and transfers
under applicable export controls, including the
[EU Dual-Use Regulation (EU) 2021/821, Articles 3–5 and Annex I](https://eur-lex.europa.eu/eli/reg/2021/821/oj/eng).
Dokime makes no controlled-technology classification or exemption claim. General
test-software availability does not classify the customer's use or evidence.

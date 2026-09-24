# BuildLore --- Project Concept

> **Turn the mess behind a project into the story behind it.**

**Status:** Concept / pre-planning\
**Working name:** BuildLore\
**Purpose:** Automatically reconstruct the journey behind a creative or
software project and turn it into useful documentation, portfolio case
studies, dev diaries, and other content.

------------------------------------------------------------------------

## 1. The Problem

Building the project is often the interesting part. Documenting it
afterwards is another project entirely.

During development, useful evidence of the process already accumulates
naturally:

-   screenshots and screen snips
-   ChatGPT conversations
-   Codex sessions and summaries
-   Git commits and diffs
-   pull requests and issues
-   README files and project notes
-   design experiments
-   error screenshots
-   abandoned approaches
-   deployment configuration
-   TODOs and roadmap ideas
-   finished screenshots and release material

Individually, these are fragments. Together, they contain the real story
of a project.

The problem is that reconstructing that story manually requires going
back through months of files, conversations, commits, screenshots, and
half-remembered decisions. By that point, documentation becomes enough
work to delay or prevent publishing the project at all.

BuildLore exists to remove that friction.

------------------------------------------------------------------------

## 2. The Core Idea

BuildLore is a local-first project archaeology and documentation tool.

The user points it at a project and gives it whatever evidence is
available:

``` text
Project/
├── repo/
├── screenshots/
├── chats/
├── notes/
└── other-context/
```

BuildLore analyses those sources, reconstructs the development history,
identifies important moments, understands relevant screenshots, and
creates a structured body of project knowledge.

That knowledge can then generate:

-   portfolio case studies
-   technical deep dives
-   dev diaries
-   build logs
-   project retrospectives
-   README content
-   launch posts
-   tutorials
-   social posts
-   project timelines

The principle is simple:

> **Document by building.**

Taking screenshots, chatting with AI, committing code, making notes,
fixing bugs, and changing direction already creates the raw material.
BuildLore turns that material into documentation.

------------------------------------------------------------------------

## 3. The Ideal User Experience

The deliberately lazy workflow:

1.  Select a project repository.
2.  Select a screenshots folder.
3.  Add conversation exports, Codex logs, notes, or other context if
    available.
4.  Click **Analyse Project**.
5.  Review the reconstructed project timeline.
6.  Click **Generate Case Study**.
7.  Make small personal edits.
8.  Publish.

The user should not need to manually organise every screenshot, remember
when every decision happened, or write a development diary while
actively developing.

BuildLore does the archaeological work afterwards.

------------------------------------------------------------------------

## 4. What BuildLore Should Understand

The goal is not simply to summarise files.

BuildLore should attempt to understand the **project journey**.

It should identify:

### Origins

-   Why did the project begin?
-   What problem was being solved?
-   What was the original scope?
-   What assumptions existed at the beginning?

### Evolution

-   How did the idea change?
-   Which features appeared later?
-   What was removed?
-   Where did the scope expand or contract?

### Design

-   UI iterations
-   UX decisions
-   visual hierarchy changes
-   branding changes
-   typography and colour decisions
-   responsive design
-   accessibility work
-   before/after states

### Engineering

-   architecture
-   technology choices
-   libraries and frameworks
-   refactoring
-   APIs
-   automation
-   deployment
-   testing
-   performance
-   security
-   maintainability

### Problems

-   bugs
-   failed approaches
-   deployment failures
-   browser issues
-   awkward technical constraints
-   design ideas that did not work

### Decisions

-   why one approach was selected over another
-   trade-offs
-   constraints
-   reversals
-   compromises
-   deliberate exclusions

### Skills Demonstrated

-   software development
-   UI/UX
-   visual design
-   branding
-   accessibility
-   SEO
-   debugging
-   DevOps
-   Git
-   automation
-   AI integration
-   research
-   problem solving
-   project management
-   documentation

### Future

-   unfinished ideas
-   planned features
-   forgotten TODOs
-   roadmap items
-   things that would be approached differently today

------------------------------------------------------------------------

## 5. Core Architecture

BuildLore should be a pipeline rather than one enormous AI prompt.

``` text
RAW SOURCES
    ↓
INGESTION
    ↓
SOURCE ANALYSIS
    ↓
NORMALISATION
    ↓
PROJECT EVENT EXTRACTION
    ↓
TIMELINE + KNOWLEDGE GRAPH
    ↓
SCREENSHOT / EVENT MATCHING
    ↓
STORY PLANNING
    ↓
CONTENT GENERATION
    ↓
MARKDOWN + SELECTED MEDIA
```

Each stage should produce reusable structured data.

This makes the system:

-   easier to debug
-   cheaper to run
-   less prone to hallucination
-   easier to extend
-   capable of incremental updates
-   independent of any single AI provider

------------------------------------------------------------------------

## 6. Input Sources

### 6.1 Git Repository

BuildLore can inspect the local repository directly.

Useful evidence includes:

-   commit history
-   commit timestamps
-   commit messages
-   diffs
-   branches
-   tags
-   release history
-   repository structure
-   dependencies
-   configuration
-   GitHub Actions
-   README
-   documentation
-   changelogs
-   source code

Git provides a strong chronological backbone.

A commit should not automatically become a paragraph in the final
article. Instead, commits provide evidence for larger development
events.

------------------------------------------------------------------------

### 6.2 Screenshots

Screenshots are one of the most valuable sources because they show the
project changing visually.

BuildLore should accept a folder containing everything --- including
messy screen snips.

For each image, vision analysis could determine:

``` json
{
  "filename": "homepage-redesign.png",
  "type": "website_screenshot",
  "project_area": "homepage",
  "description": "Dark portfolio homepage featuring a recently shipped project",
  "visible_text": [
    "Recently shipped",
    "View source"
  ],
  "development_stage": "late",
  "likely_purpose": "design review",
  "notable_elements": [
    "video preview",
    "project metadata",
    "CTA hierarchy"
  ],
  "story_relevance": [
    "homepage redesign",
    "visual hierarchy",
    "project promotion"
  ]
}
```

The system should understand the image visually rather than treating
screenshots purely as OCR targets.

------------------------------------------------------------------------

### 6.3 Screenshot Metadata

Useful chronological clues can come from:

-   filename timestamps
-   creation dates
-   modified dates
-   EXIF data where available
-   nearby Git activity
-   matching conversation dates

No single timestamp source should be treated as infallible.

BuildLore can combine them to estimate where an image belongs in the
project timeline.

------------------------------------------------------------------------

### 6.4 ChatGPT Conversations

Chat conversations often contain something Git cannot provide:

**why a change happened.**

A Git commit might say:

> Refine projects layout

The related conversation might explain:

> The page feels too narrow, the shared secondary-page layout is
> restricting it, and the hierarchy makes the project feel less
> important.

That reasoning is valuable portfolio material.

BuildLore should support imported conversation data rather than
depending on permanent direct access to a particular chat platform.

Possible ingestion methods:

-   Markdown exports
-   text exports
-   ChatGPT account exports
-   copied conversations
-   project summaries
-   manually saved transcripts

------------------------------------------------------------------------

### 6.5 Codex Sessions

Codex is particularly useful because its summaries often document:

-   files changed
-   implementation details
-   tests performed
-   bugs discovered
-   technical reasoning
-   commits created
-   limitations

BuildLore should convert these into project events rather than simply
reproducing Codex summaries.

------------------------------------------------------------------------

### 6.6 Notes

The tool should also accept loose Markdown or text notes.

These may contain:

-   original ideas
-   TODOs
-   design thoughts
-   feature ideas
-   frustrations
-   roadmap plans
-   abandoned concepts

Messy notes are expected.

The tool exists partly so the user does **not** have to maintain
immaculate documentation while building.

------------------------------------------------------------------------

## 7. Project Events

The central internal concept should be a **Project Event**.

Instead of storing a project as a pile of chats and commits, BuildLore
transforms evidence into events.

Example:

``` json
{
  "date": "2026-09-10",
  "type": "design_change",
  "title": "Redesigned the shipped-project feature",
  "summary": "The existing presentation felt fragmented and contained excessive whitespace.",
  "decision": "Replace it with a unified editorial feature layout.",
  "reasoning": [
    "improve visual hierarchy",
    "increase project prominence",
    "reduce unnecessary spacing"
  ],
  "evidence": [
    "chat:192",
    "commit:6d782a5",
    "image:388"
  ],
  "confidence": "high"
}
```

Suggested event categories:

``` text
idea
research
decision
design_change
feature
bug
problem
fix
refactor
experiment
deployment
release
milestone
abandoned_idea
lesson
future_plan
```

------------------------------------------------------------------------

## 8. Evidence and Confidence

BuildLore should avoid pretending that every AI inference is a fact.

Every significant claim should be traceable to evidence.

Evidence could include:

``` text
CHAT
COMMIT
DIFF
CODE
IMAGE
NOTE
ISSUE
PULL REQUEST
RELEASE
```

Events should carry a confidence level:

``` text
confirmed
high confidence
probable
inferred
uncertain
```

The final writer can then avoid presenting weak inference as established
history.

This is particularly important when reconstructing older projects.

------------------------------------------------------------------------

## 9. Screenshot Intelligence

Screenshot handling should be a major feature rather than an attachment
afterthought.

### Analyse Everything, Publish Selectively

The user can dump 100 screenshots into the tool.

BuildLore might determine:

``` text
100 images supplied
↓
82 relevant to project
↓
41 meaningfully unique
↓
15 historically interesting
↓
6 recommended for article
```

The final case study should not become a screenshot graveyard.

------------------------------------------------------------------------

## 10. Screenshot Ranking

Each image could receive internal scores such as:

``` text
Visual quality          8/10
Historical importance   9/10
Uniqueness               7/10
Narrative usefulness     9/10
Duplicate probability    2/10
```

Additional factors:

-   readable resolution
-   whether it shows a meaningful state
-   whether another screenshot communicates the same thing better
-   whether it demonstrates a before/after transition
-   whether it supports an important project event
-   whether it contains sensitive or irrelevant information

------------------------------------------------------------------------

## 11. Detecting Visual Evolution

BuildLore should attempt to identify screenshots showing different
versions of the same interface.

Example:

``` text
homepage-early.png
homepage-redesign.png
homepage-final.png
```

Vision analysis could group these as:

> Homepage evolution

The writer could then automatically construct:

``` md
## Evolving the Homepage

### Early Direction

![Early homepage](...)

The original homepage...

### Refinement

![Intermediate homepage](...)

After reviewing the hierarchy...

### Current Direction

![Current homepage](...)

The final iteration...
```

This transforms random old screenshots into evidence of design thinking.

------------------------------------------------------------------------

## 12. Cross-Source Contextualisation

The strongest feature will come from connecting evidence.

For example:

``` text
Screenshot
12 August
↓
Shows cramped project cards

Chat
13 August
↓
"I don't like how narrow this feels"

Commit
13 August
↓
"Create dedicated projects shell"

Screenshot
14 August
↓
Shows wider redesigned layout
```

BuildLore can reconstruct this as one coherent development event.

This is substantially more useful than independently summarising the
screenshot, conversation, and commit.

------------------------------------------------------------------------

## 13. The Project Timeline

After ingestion, BuildLore should produce a browsable chronological
timeline.

Example:

``` text
APR 12
Original concept created

APR 15
First functional prototype

APR 18
Initial UI abandoned

APR 21
New component architecture introduced

MAY 02
Deployment failure discovered

MAY 03
Cloudflare configuration fixed

MAY 09
Accessibility pass

MAY 15
First public release
```

Selecting an event could reveal:

-   explanation
-   associated screenshots
-   related chats
-   commits
-   files
-   decisions
-   confidence

This timeline is useful even without generating a blog.

------------------------------------------------------------------------

## 14. Project Archaeology

BuildLore should actively search for forgotten material.

A dedicated **Project Archaeology** report could identify:

-   abandoned ideas
-   features discussed but never implemented
-   forgotten TODOs
-   design directions later reversed
-   repeated problems
-   ideas that eventually resurfaced
-   features that quietly disappeared
-   decisions that proved successful
-   decisions later undone

This turns the tool into something useful for continuing development as
well as documenting it.

------------------------------------------------------------------------

## 15. Story Planning

The final article should **not** be a changelog.

Before writing, BuildLore should construct a narrative plan.

For example:

``` text
1. The problem
2. The original idea
3. Building the first prototype
4. Discovering the main limitation
5. Rethinking the design
6. Solving the difficult technical problem
7. Preparing the release
8. What the project taught me
9. Where it goes next
```

Only the events that contribute to the story should appear prominently.

Minor commits remain supporting evidence.

------------------------------------------------------------------------

## 16. Generated Case Study

The primary output should be clean Markdown suitable for a
portfolio/blog.

Example:

``` md
---
title: "Building ClipSift"
description: "How I built an AI-assisted CCTV review tool..."
date: 2026-09-10
tags:
  - AI
  - Python
  - Gemma
---

# Building ClipSift

## The Problem

...

## From CLI Experiment to Working Tool

...

![Early interface](./images/early-interface.png)

## Rethinking the Workflow

...

![Updated interface](./images/final-interface.png)

## What I Learned

...
```

Frontmatter should eventually be configurable for different websites.

------------------------------------------------------------------------

## 17. Writing Principles

Generated articles should prioritise:

-   reasoning over feature lists
-   process over self-promotion
-   specific examples over generic claims
-   problems and solutions
-   visible evolution
-   genuine technical detail
-   trade-offs
-   lessons
-   evidence

Avoid empty portfolio language such as:

> This project demonstrates my strong problem-solving abilities.

Prefer showing the evidence:

> The original layout reused the site's secondary-page container, but
> its 44rem content limit made visual project material feel cramped. I
> eventually separated the Projects route into its own 72rem shell
> rather than continuing to fight the shared cascade.

The reader can infer the skill.

------------------------------------------------------------------------

## 18. "Things Nobody Sees"

A useful optional article section could expose the invisible work behind
a polished project:

-   hours debugging a strange deployment problem
-   features removed before release
-   failed visual directions
-   browser quirks
-   accessibility fixes
-   refactors
-   performance work
-   awkward API limitations
-   Git mistakes
-   deployment configuration

These details make a case study feel like a genuine development story
rather than marketing copy.

------------------------------------------------------------------------

## 19. Output Modes

The same analysed project knowledge could generate multiple forms of
content.

### Portfolio

**Portfolio Case Study**\
Polished narrative focusing on process, decisions, skills, and outcome.

### Development

**Technical Deep Dive**\
Architecture, implementation, performance, testing, and engineering
decisions.

**Dev Diary**\
More chronological and personal.

**Build Log**\
Concise timeline of major development events.

**Project Retrospective**\
What worked, what failed, and what would change next time.

### Documentation

**README**\
Repository-focused overview.

**Architecture Notes**\
Technical system explanation.

**Roadmap Recovery**\
Forgotten ideas and unfinished work.

### Content

**Launch Post**

**Blog Post**

**LinkedIn Post**

**X Thread**

**Video Outline / Script**

The expensive project analysis happens once; many outputs can reuse it.

------------------------------------------------------------------------

## 20. Persistent Project Knowledge

BuildLore should create a hidden/local project knowledge directory.

Possible structure:

``` text
.buildlore/
├── project.json
├── knowledge.json
├── timeline.json
├── events.json
├── screenshots.json
├── sources.json
├── decisions.json
├── features.json
└── cache/
```

Generated content could live separately:

``` text
buildlore-output/
├── case-study.md
├── project-history.md
├── archaeology.md
├── skills.md
└── images/
```

------------------------------------------------------------------------

## 21. Incremental Analysis

A major design goal should be avoiding complete re-analysis.

Example:

``` text
Last analysis:
1 September 2026

New material:
12 commits
17 screenshots
3 Codex transcripts
2 notes
```

BuildLore analyses only the additions, then updates the project
knowledge.

Benefits:

-   lower API cost
-   faster processing
-   better local-model viability
-   persistent understanding
-   useful throughout development rather than only at the end

------------------------------------------------------------------------

## 22. Local-First AI Strategy

Not every task requires an expensive reasoning model.

A possible model pipeline:

``` text
LOCAL / CHEAP MODEL
    ↓
classification
chunking
metadata extraction
basic event candidates
duplicate detection

VISION MODEL
    ↓
screenshot understanding
visual grouping
before/after detection

STRONG REASONING MODEL
    ↓
cross-source reconstruction
decision extraction
timeline interpretation
story planning

STRONG WRITING MODEL
    ↓
final case study
technical article
other publication outputs
```

This allows BuildLore to balance:

-   privacy
-   quality
-   speed
-   token usage
-   API cost

Provider abstraction should be considered early so the application is
not tightly coupled to one model vendor.

------------------------------------------------------------------------

## 23. Possible Desktop UI

A desktop application suits the workflow because BuildLore needs local
filesystem and Git access.

Concept:

``` text
┌─────────────────────────────────────────────────────────────┐
│ BUILDLORE                                                   │
│ Turn the mess behind a project into the story behind it.   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ PROJECT                                                     │
│ C:\GitHub\ClipSift                                [Browse]   │
│                                                             │
│ SOURCES                                                     │
│ ✓ Git repository                                            │
│ ✓ Screenshots                              73 images         │
│ ✓ Chat exports                             14 conversations  │
│ ✓ Notes                                     8 files          │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ PROJECT KNOWLEDGE                                           │
│                                                             │
│ 124  Development events                                     │
│  31  Decisions                                              │
│  17  Problems / fixes                                       │
│  11  Abandoned ideas                                        │
│  18  Interesting screenshots                                │
│                                                             │
│ [Explore Timeline]        [Generate Case Study]              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Potential sections:

``` text
Overview
Timeline
Screenshots
Decisions
Problems
Features
Skills
Archaeology
Sources
Generate
Settings
```

------------------------------------------------------------------------

## 24. Human Review

BuildLore should automate the boring work without removing authorship.

Before publishing, the user should be able to:

-   exclude incorrect events
-   correct dates
-   mark an inference as confirmed
-   hide screenshots
-   pin important screenshots
-   edit generated captions
-   choose story emphasis
-   add personal context
-   regenerate individual sections

The ideal outcome is:

> **90% automatically reconstructed, 10% human personality and
> correction.**

Not:

> AI writes an unquestioned biography of the repository.

------------------------------------------------------------------------

## 25. Privacy

Projects may contain:

-   private source code
-   API details
-   screenshots of desktop environments
-   personal conversations
-   tokens or credentials accidentally visible in screenshots
-   private Git history

BuildLore should therefore be designed with privacy in mind.

Potential safeguards:

-   local-first indexing
-   explicit control over cloud model usage
-   ignore patterns
-   `.gitignore`-style exclusion rules
-   secret detection before cloud submission
-   screenshot privacy warnings
-   source preview before analysis
-   never upload entire repositories when only selected context is
    required

------------------------------------------------------------------------

## 26. Possible Technology Direction

This should remain open during planning, but a sensible direction could
be:

### Desktop

-   Tauri
-   React
-   TypeScript
-   Vite

### Local backend

-   Rust commands through Tauri and/or a small local processing layer
-   direct filesystem access
-   Git CLI or Git library

### Data

-   JSON for initial prototype
-   SQLite once relationships/search become more complex

### AI

-   provider abstraction
-   optional local Ollama-compatible models
-   cloud reasoning/vision providers where useful
-   structured JSON responses validated against schemas

### Markdown

-   generated `.md` files
-   configurable frontmatter
-   copied/selected screenshots
-   relative image paths

The first prototype should prioritise proving the reconstruction
pipeline rather than building a large polished UI.

------------------------------------------------------------------------

## 27. MVP

The first version does **not** need every planned integration.

### MVP Inputs

-   local Git repository
-   screenshots folder
-   Markdown/text conversation exports
-   Markdown/text notes

### MVP Analysis

-   inspect Git history
-   analyse screenshots
-   extract project events from conversations
-   merge events chronologically
-   associate screenshots with events
-   identify major decisions/problems
-   rank screenshots

### MVP Outputs

``` text
project-history.md
timeline.json
screenshot-index.json
case-study.md
```

### MVP UI

``` text
Select project
Select screenshot folder
Add context files
Analyse
Review timeline
Generate
```

If this works well on one real project, the concept is proven.

------------------------------------------------------------------------

## 28. What Not to Build First

Avoid immediately adding:

-   every AI provider
-   GitHub OAuth
-   automatic publishing
-   CMS integrations
-   elaborate knowledge graphs
-   social media integrations
-   team collaboration
-   cloud accounts
-   complex theming
-   dozens of article modes

The difficult and valuable question is:

> **Can BuildLore correctly reconstruct why a project changed and
> produce a case study that feels like the creator actually remembers
> building it?**

Solve that first.

------------------------------------------------------------------------

## 29. First Real Test

A strong first test would use a project with:

-   substantial Git history
-   multiple visual iterations
-   screenshots
-   ChatGPT discussion
-   Codex implementation summaries
-   deployment work
-   clear technical challenges
-   a finished or near-finished outcome

Run BuildLore against the material without manually organising it first.

That is important.

If the user has to spend an hour preparing the evidence for the
automation tool, the tool has partially failed its purpose.

------------------------------------------------------------------------

## 30. Success Criteria

The project is successful if:

1.  A messy folder can be used without extensive preparation.
2.  BuildLore reconstructs a recognisable project timeline.
3.  It identifies meaningful decisions rather than merely listing
    commits.
4.  It connects conversations with implementation evidence.
5.  It understands what screenshots show.
6.  It identifies useful before/after visual states.
7.  It ignores most irrelevant screenshots.
8.  The generated article accurately reflects the development journey.
9.  Technical claims are grounded in evidence.
10. The resulting Markdown needs editing for personality, not complete
    rewriting.
11. Re-running the tool primarily analyses new material.
12. The output is useful enough to actually publish.

------------------------------------------------------------------------

## 31. Longer-Term Possibilities

Once the core works, BuildLore could evolve into a broader
project-memory system.

Potential ideas:

-   GitHub issue and PR ingestion
-   automatic release-history reconstruction
-   semantic project search
-   "Why did I build this this way?" queries
-   forgotten-feature recovery
-   automatic changelogs
-   development statistics
-   visual evolution galleries
-   project health reports
-   release-note generation
-   portfolio-site integration
-   static-site adapters
-   automatic image optimisation
-   caption and alt-text generation
-   video script generation from the same project history
-   cross-project skill analysis
-   portfolio-wide technology/skill mapping

A particularly useful query could eventually be:

> "Why did I stop using the shared secondary layout on this page?"

BuildLore could answer using the original conversation, commit, and
screenshots rather than relying on memory.

------------------------------------------------------------------------

## 32. Product Philosophy

BuildLore should be designed around a slightly chaotic but realistic
creative workflow.

The user should not need to change how they work in order to document
their work.

They can continue to:

-   take random screenshots
-   have long ChatGPT conversations
-   use Codex to implement changes
-   make Git commits
-   write occasional notes
-   change their mind
-   abandon ideas
-   revisit projects later

Those behaviours are not noise to be eliminated.

They are the project's **lore**.

BuildLore's job is to turn that lore into something structured, useful,
searchable, and publishable.

------------------------------------------------------------------------

## 33. One-Sentence Vision

> **BuildLore automatically reconstructs the story of how a project was
> built from the digital trail left behind while building it.**

------------------------------------------------------------------------

## 34. Short Pitch

**BuildLore** is a local-first AI project archaeology tool that analyses
Git history, screenshots, AI conversations, development notes, and other
project evidence to reconstruct the complete development journey.

Instead of manually writing a case study months later, creators can
point BuildLore at the mess they naturally accumulated while working. It
builds a timeline, identifies decisions and challenges, understands
visual iterations, selects meaningful screenshots, and generates
evidence-backed Markdown documentation.

**Build first. The documentation is already happening.**

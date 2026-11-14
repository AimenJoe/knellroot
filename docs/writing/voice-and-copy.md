# Knellroot — Voice & Copy Guide

- Status: **CANON for editorial practice** (does not add or alter world facts)
- Scope: all player-facing Korean and English text, including dialogue, objectives, tutorials, codex, equipment, menus, prompts, and notifications
- Lore boundary: this guide follows the established canon and mysteries. It is not permission to explain unresolved lore.

## 1. The voice of the game

Knellroot is a dark-fantasy mystery told through a small number of words and a great deal of place, sound, and action.

- **Show before telling.** Let a place, object, repeated action, or change in the world carry meaning before a character explains it.
- **Keep the wanderer unvoiced.** Use restrained narration, not first-person thoughts, questions, or a chatty narrator. The opening's "Eyes open" / "The body remembers" is the reference.
- **Do not explain the mystery.** Never turn the bell, the roots, the core, the wanderer's past, the village's departure, or the shrine's builders into settled explanations unless a later canon decision explicitly does so.
- **Leave room, not vagueness.** A line may suggest an image or feeling, but it must not hide what the player needs to do or what a system has changed.
- **Prefer concrete words.** Fire, stone, water, cloth, root, bell, hand, breath. Avoid stacking abstract fantasy nouns or ornate metaphors.
- **Use silence.** Not every object needs a caption; not every discovery needs a toast; not every scene needs dialogue.

## 2. Voice by text type

| Type | Voice | Rule |
|---|---|---|
| Opening / scene narration | Sparse, observational, present tense | No first-person voice; one image or fact per line |
| Residents | Brief, ordinary, self-contained | Talk about the task at hand; never explain history, lore, quests, or the player's identity |
| Objectives | Clear imperative or destination | One action per line; use the same verb as the interaction when possible |
| Tutorial hints | Body-led, restrained | The action name stays explicit; the smaller line may evoke bodily memory but must not replace the instruction |
| Codex observations | Concrete, observable | Describe only what the player could see or test |
| Codex guesses | Tentative | Use "듯하다", "알 수 없다", "seems", "remains unknown"; never promote a guess into fact |
| Item flavor | One tactile image, a trace of use | Do not invent owners, dates, motives, or supernatural properties |
| Stats / equipment | Exact and compact | Mechanics, units, and trade-offs take priority over atmosphere |
| Menus / confirmations | Calm and unambiguous | State what is lost, kept, spent, or restored before asking |
| Toasts | Short event report | Say what changed; avoid celebratory voice and redundant explanation |

## 3. Korean style

- Default narration uses concise present-tense statements: "눈을 뜬다", "몸만이 무언가를 기억한다".
- Residents use natural spoken Korean and contractions where they fit, but do not become cute, comic, or overly familiar.
- Do not force every line into archaic or literary endings. A resident can say "아직 쓸 만해"; an objective can say "…향하라".
- Keep gameplay terms consistent with the existing UI glossary: **기력**, **등잔**, **잔불**, **쉼터**, **화로**, **장신구**, **도감**, **덧옷**, **패링**.
- Use **핵** for the in-world core. Do not replace it with a more explanatory term.
- Avoid mixing English loanwords and native terms for the same stat in adjacent screens. Where a system label is already established, preserve it rather than inventing a poetic synonym.
- Use punctuation to separate distinct thoughts. Avoid ellipses as decoration; reserve them for an actual trailing thought or interruption.

## 4. English style

- Use the same restrained, observational distance as Korean. The wanderer is not a chatty "you".
- Scene narration favors short present-tense fragments: "Eyes open. No name." Avoid explanatory clauses when a short sentence works.
- Residents may use contractions and plain speech. Their English should sound like ordinary people occupied with ordinary work, not translated lore.
- Objectives use direct verbs: "Find", "Enter", "Descend", "Take". Never sacrifice clarity for a poetic synonym.
- Use established terms consistently: **hearth**, **rest site**, **lamp**, **embers**, **core**, **charm**, **codex**, **poise**, **stagger**. Do not alternate between "flask" and "lamp" for the same item.
- Avoid "you" in scene narration and tutorial flavor. It remains acceptable in a direct UI instruction only when needed for clarity.
- Do not translate Korean word order literally. Preserve intent, register, and uncertainty rather than syntax.

## 5. Canon and mystery guardrails

- The three bells remain distinct in the text unless canon says otherwise: the village's broken bell, the silent bell visible at the sanctuary, and the bell heard beneath the earth.
- The Broken Bellkeeper's behavior is an observation. Its name and actions do not prove whether it rang or guarded the village bell.
- Residents are not historians. Their lines do not reveal why the village was left or who built the sanctuary.
- The shrine-village connection remains **PROPOSED**. Similar stonework, vertical routes, and bell customs may invite comparison, but no line may state that the same people built both places.
- The roots may be described as present, growing, or physically connecting spaces. Do not assign them intention, judgment, or a plan.
- Codex guesses remain visibly and grammatically distinct from observations.

## 6. Mechanical clarity and localization

- Never remove a control name, count, cost, duration, target, or consequence to make a line more atmospheric.
- Preserve every format placeholder (`{0}`, `{1}`, etc.) and its order unless the formatter and call site are changed together.
- Korean and English must communicate the same gameplay effect, not necessarily use the same number of words.
- Keep short HUD labels short. Put flavor in the description field, not in a button label.
- When a term changes, update both languages and check every related use before merging.
- Check text fit at the supported desktop and landscape-phone sizes; a string that reads well in a source file may still obscure the game.

## 7. Review checklist

- Does the line reveal a fact that is still **PROPOSED** or **MYSTERY**?
- Does it sound like this speaker, or like a narrator explaining the world?
- Can the player still understand the action, cost, result, and risk immediately?
- Is an observation being presented as a guess, or vice versa?
- Do Korean and English preserve the same meaning, uncertainty, and emotional distance?
- Are established terms and placeholders unchanged?
- Does the text fit in its actual UI surface?

import * as monaco from "monaco-editor";
import { tokenize } from "../bridge";

export const foldProvider = monaco.languages.registerFoldingRangeProvider("wikidot", {
    provideFoldingRanges(
        model: monaco.editor.ITextModel,
    ): monaco.languages.FoldingRange[] {
        const { tokens } = tokenize(model.getValue());
        const ranges: monaco.languages.FoldingRange[] = [];
        let startLine: number | undefined;

        for (const token of tokens) {
            const mark = token["include-fold-mark"];
            if (mark === "include-start") {
                startLine = model.getPositionAt(token.span.start).lineNumber;
            } else if (mark === "include-end" && startLine !== undefined) {
                const endLine = model.getPositionAt(token.span.start).lineNumber;
                if (endLine > startLine) ranges.push({ start: startLine, end: endLine });
                startLine = undefined;
            }
        }
        return ranges;
    },
});

if (import.meta.hot) {
    import.meta.hot.dispose(() => foldProvider.dispose());
}

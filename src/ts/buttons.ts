export interface WrapButton {
  label: string;
  open: string;
  close: string;
}

export const BUTTONS: WrapButton[] = [
  { label: "B", open: "**", close: "**" },
  { label: "I", open: "//", close: "//" },
  { label: "U", open: "__", close: "__" },
  { label: "S", open: "--", close: "--" },
  { label: "x²", open: "^^", close: "^^" },
  { label: "x₂", open: ",,", close: ",," },
  { label: "{ }", open: "{{", close: "}}" },
];

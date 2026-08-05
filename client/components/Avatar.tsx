import { colorForUsername, initialsForUsername } from "@/lib/avatar";

const SIZE_CLASSES = {
  sm: "w-6 h-6 text-xs",
  md: "w-9 h-9 text-sm",
  lg: "w-16 h-16 text-xl",
};

export function Avatar({
  username,
  size = "md",
}: {
  username: string;
  size?: "sm" | "md" | "lg";
}) {
  const color = colorForUsername(username);
  const initials = initialsForUsername(username);

  return (
    <div
      className={`rounded-full flex items-center justify-center font-semibold text-white shrink-0 ${SIZE_CLASSES[size]}`}
      style={{ backgroundColor: color }}
      title={username}
    >
      {initials}
    </div>
  );
}
import * as React from "react";
import { cn } from "../../lib/utils";

const variants: Record<string, string> = {
  default: "bg-primary text-primary-foreground hover:bg-primary/90",
  outline: "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
  ghost: "hover:bg-accent hover:text-accent-foreground",
  secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
  destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
  link: "text-primary underline-offset-4 hover:underline",
};

const sizes: Record<string, string> = {
  default: "h-11 px-4 text-sm md:h-8 md:px-3",
  sm: "h-9 px-3 text-xs md:h-7 md:px-2",
  icon: "h-11 w-11 p-0 md:h-8 md:w-8",
};

export function Button({ className, variant = "default", size = "default", ...props }: React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: keyof typeof variants; size?: keyof typeof sizes }) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-1.5 rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 min-h-[44px] min-w-[44px] md:min-h-0 md:min-w-0",
        variants[variant],
        sizes[size],
        className
      )}
      {...props}
    />
  );
}

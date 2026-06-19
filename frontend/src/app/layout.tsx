import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Soroban Wave Stake Protocol",
  description: "Anti-Ghosting Micro-Staking Protocol for Decentralized Engineering Sprints",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased bg-[#0d1117] text-[#c9d1d9]">{children}</body>
    </html>
  );
}

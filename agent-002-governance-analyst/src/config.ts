export const config = {
  // Regen LCD endpoint
  lcdUrl: process.env.REGEN_LCD_URL || "https://regen.api.chandrastation.com",

  // Anthropic
  anthropicApiKey: process.env.ANTHROPIC_API_KEY || "",
  model: process.env.ANTHROPIC_MODEL || "claude-sonnet-4-5-20250929",

  // Discord webhook (optional)
  discordWebhookUrl: process.env.DISCORD_WEBHOOK_URL || "",

  // Polling
  pollIntervalMs: (parseInt(process.env.POLL_INTERVAL_SECONDS || "300", 10)) * 1000,

  // Governance parameters (Regen Network defaults)
  governance: {
    quorumThreshold: 0.334,   // 33.4%
    passThreshold: 0.5,        // 50% of Yes/(Yes+No+NoWithVeto)
    vetoThreshold: 0.334,      // 33.4% of NoWithVeto/Total
    votingPeriodSeconds: 604_800, // 7 days
  },

  // Agent identity
  agentId: "AGENT-002",
  agentName: "RegenGovernanceAnalyst",
  governanceLayer: 1 as const,
} as const;

export function validateConfig(): void {
  if (!config.anthropicApiKey) {
    throw new Error(
      "ANTHROPIC_API_KEY is required. Copy .env.example to .env and set it."
    );
  }
}

export enum SlashCommandAvailability {
  AnyMessage = "ANY_MESSAGE",
  FollowUpOnly = "FOLLOW_UP_ONLY",
}

export type SlashCommand = {
  name: string;
  description: string;
  availability: SlashCommandAvailability;
};

export const SLASH_COMMANDS: SlashCommand[] = [
  {
    name: "canvas",
    description: "Generate canvas for the conversation",
    availability: SlashCommandAvailability.FollowUpOnly,
  },
];

export const SLASH_COMMANDS_FOR_FIRST_MESSAGE: SlashCommand[] =
  SLASH_COMMANDS.filter(
    (command) => command.availability === SlashCommandAvailability.AnyMessage,
  );

export const findMatchingSlashCommands = (
  availableCommands: SlashCommand[],
  query: string,
): SlashCommand[] => {
  const normalizedQuery = query.toLowerCase();
  return availableCommands.filter(
    (command) =>
      command.name.toLowerCase().includes(normalizedQuery) ||
      command.description.toLowerCase().includes(normalizedQuery),
  );
};

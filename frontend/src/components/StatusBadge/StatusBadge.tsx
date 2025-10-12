import StatusBadgeStyles from "src/components/StatusBadge/StatusBadge.module.css";
import { StatusType } from "src/components/Connection/dto";

interface StatusBadgeProps {
  status: StatusType;
  text: string;
}

export function StatusBadge({ status, text }: StatusBadgeProps) {
  let statusSpecificClass = StatusBadgeStyles.default;

  switch (status) {
    case StatusType.Active:
      statusSpecificClass = StatusBadgeStyles.active;
      break;
    case StatusType.Inactive:
      statusSpecificClass = StatusBadgeStyles.inactive;
      break;
    case StatusType.Failed:
      statusSpecificClass = StatusBadgeStyles.failed;
      break;
  }

  return (
    <div className={`${StatusBadgeStyles.base} ${statusSpecificClass}`}>
      {text}
    </div>
  );
}

import { copy } from "./content/copy";

type TimeControlsProps = {
  yearsToAdvance: number;
  yearStepOptions: readonly number[];
  onYearsToAdvanceChange: (years: number) => void;
  onAdvanceOneYear: () => void;
  onAdvanceYears: () => void;
};

export function TimeControls({
  yearsToAdvance,
  yearStepOptions,
  onYearsToAdvanceChange,
  onAdvanceOneYear,
  onAdvanceYears,
}: TimeControlsProps) {
  return (
    <section className="controls" aria-label={copy.timeControlsLabel}>
      <button className="button" type="button" onClick={onAdvanceOneYear}>
        {copy.advanceOneYear}
      </button>
      <label className="control-field" htmlFor="years-to-advance">
        {copy.advanceYearsLabel}
        <select
          id="years-to-advance"
          value={yearsToAdvance}
          onChange={(event) => onYearsToAdvanceChange(Number(event.target.value))}
        >
          {yearStepOptions.map((years) => (
            <option key={years} value={years}>
              {years}
            </option>
          ))}
        </select>
      </label>
      <button className="button" type="button" onClick={onAdvanceYears}>
        {copy.advanceYearsButton}
      </button>
    </section>
  );
}

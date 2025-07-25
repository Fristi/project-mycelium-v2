import { ExclamationCircleIcon } from "@heroicons/react/20/solid";
import { FieldHookConfig, useField } from "formik";

type Option = {
  value: string;
  label: string;
};

type Props = {
  label: string;
  options: Option[];
  helperText?: string;
};

const Select = (props: Props & FieldHookConfig<string>) => {
  const [field, meta] = useField<string>(props.name);
  const isError = !!meta.error && meta.touched;

  return (
    <div className="mt-6 sm:mt-5">
      <div className="sm:grid sm:grid-cols-3 sm:gap-4 sm:items-start sm:border-t sm:border-gray-200 sm:pt-5">
        <label className="block text-sm font-medium text-gray-700">{props.label}</label>
        <div className="mt-1 relative rounded-md shadow-sm">
          <select
            {...field}
            className="mt-2 block w-full rounded-md border-0 py-1.5 pl-3 pr-10 text-gray-900 ring-1 ring-inset ring-gray-300 focus:ring-2 focus:ring-indigo-600 sm:text-sm sm:leading-6"
          >
            {props.options.map((opt) => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>

          {isError && (
            <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
              <ExclamationCircleIcon className="h-5 w-5 text-red-500" aria-hidden="true" />
            </div>
          )}
        </div>
        {isError && <p className="mt-2 text-sm text-red-600">{props.helperText}</p>}
      </div>
    </div>
  );
};

export default Select;

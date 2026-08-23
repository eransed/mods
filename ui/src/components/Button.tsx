import type { ButtonHTMLAttributes } from 'react';

export function Button({ className = '', ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
    return <button className={`standard-button ${className}`.trim()} {...props} />;
}
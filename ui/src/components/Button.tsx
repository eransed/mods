import type { ButtonHTMLAttributes } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary';
}

export function Button({ className = '', variant = 'secondary', ...props }: ButtonProps) {
    return <button className={`standard-button standard-button-${variant} ${className}`.trim()} {...props} />;
}
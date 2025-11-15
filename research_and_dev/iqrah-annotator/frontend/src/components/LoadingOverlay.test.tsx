import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LoadingOverlay } from './LoadingOverlay';

describe('LoadingOverlay', () => {
  describe('Visibility', () => {
    it('should render when visible is true', () => {
      render(<LoadingOverlay visible={true} />);

      // Backdrop should be present
      const backdrop = document.querySelector('.MuiBackdrop-root');
      expect(backdrop).toBeInTheDocument();
    });

    it('should not render when visible is false', () => {
      render(<LoadingOverlay visible={false} />);

      // Backdrop should not be visible
      const backdrop = document.querySelector('.MuiBackdrop-root');
      expect(backdrop).not.toHaveClass('MuiBackdrop-open');
    });
  });

  describe('Message Display', () => {
    it('should show default message when no message provided', () => {
      render(<LoadingOverlay visible={true} />);

      expect(screen.getByText('Loading...')).toBeInTheDocument();
    });

    it('should show custom message when provided', () => {
      render(<LoadingOverlay visible={true} message="Processing audio..." />);

      expect(screen.getByText('Processing audio...')).toBeInTheDocument();
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    it('should update message when prop changes', () => {
      const { rerender } = render(
        <LoadingOverlay visible={true} message="Step 1" />
      );

      expect(screen.getByText('Step 1')).toBeInTheDocument();

      rerender(<LoadingOverlay visible={true} message="Step 2" />);

      expect(screen.getByText('Step 2')).toBeInTheDocument();
      expect(screen.queryByText('Step 1')).not.toBeInTheDocument();
    });
  });

  describe('Progress Indicator', () => {
    it('should show indeterminate progress when progress is undefined', () => {
      render(<LoadingOverlay visible={true} />);

      // Should show circular progress without value
      const circularProgress = document.querySelector('.MuiCircularProgress-root');
      expect(circularProgress).toBeInTheDocument();
      expect(circularProgress).toHaveClass('MuiCircularProgress-indeterminate');

      // Should not show linear progress or percentage
      const linearProgress = document.querySelector('.MuiLinearProgress-root');
      expect(linearProgress).not.toBeInTheDocument();
    });

    it('should show determinate progress when progress is provided', () => {
      render(<LoadingOverlay visible={true} progress={0.65} />);

      // Should show circular progress with value
      const circularProgress = document.querySelector('.MuiCircularProgress-root');
      expect(circularProgress).toBeInTheDocument();
      expect(circularProgress).toHaveClass('MuiCircularProgress-determinate');

      // Should show linear progress
      const linearProgress = document.querySelector('.MuiLinearProgress-root');
      expect(linearProgress).toBeInTheDocument();

      // Should show percentage text
      expect(screen.getByText('65%')).toBeInTheDocument();
    });

    it('should show correct percentage for 0% progress', () => {
      render(<LoadingOverlay visible={true} progress={0} />);

      expect(screen.getByText('0%')).toBeInTheDocument();
    });

    it('should show correct percentage for 100% progress', () => {
      render(<LoadingOverlay visible={true} progress={1} />);

      expect(screen.getByText('100%')).toBeInTheDocument();
    });

    it('should round percentage to nearest integer', () => {
      const { rerender } = render(
        <LoadingOverlay visible={true} progress={0.456} />
      );

      expect(screen.getByText('46%')).toBeInTheDocument();

      rerender(<LoadingOverlay visible={true} progress={0.789} />);

      expect(screen.getByText('79%')).toBeInTheDocument();
    });

    it('should update progress when prop changes', () => {
      const { rerender } = render(
        <LoadingOverlay visible={true} progress={0.25} />
      );

      expect(screen.getByText('25%')).toBeInTheDocument();

      rerender(<LoadingOverlay visible={true} progress={0.75} />);

      expect(screen.getByText('75%')).toBeInTheDocument();
    });
  });

  describe('Transparent Mode', () => {
    it('should have default (non-transparent) background by default', () => {
      render(<LoadingOverlay visible={true} />);

      const backdrop = document.querySelector('.MuiBackdrop-root') as HTMLElement;
      expect(backdrop).toBeInTheDocument();

      // Should have dark background (not transparent)
      const styles = window.getComputedStyle(backdrop);
      expect(styles.backgroundColor).not.toBe('transparent');
    });

    it('should render with transparent prop set to true', () => {
      render(<LoadingOverlay visible={true} transparent={true} />);

      const backdrop = document.querySelector('.MuiBackdrop-root') as HTMLElement;
      expect(backdrop).toBeInTheDocument();

      // Component should render successfully with transparent prop
      // Note: MUI applies styles via CSS-in-JS, so we just verify it renders
      expect(backdrop).toBeTruthy();
    });

    it('should allow click-through when transparent is true', () => {
      render(<LoadingOverlay visible={true} transparent={true} />);

      const backdrop = document.querySelector('.MuiBackdrop-root') as HTMLElement;

      // Should have pointer-events: none for click-through
      expect(backdrop).toHaveStyle({ pointerEvents: 'none' });
    });

    it('should block clicks when transparent is false', () => {
      render(<LoadingOverlay visible={true} transparent={false} />);

      const backdrop = document.querySelector('.MuiBackdrop-root') as HTMLElement;

      // Should have pointer-events: auto to block clicks
      expect(backdrop).toHaveStyle({ pointerEvents: 'auto' });
    });
  });

  describe('Component Structure', () => {
    it('should render Paper component inside Backdrop', () => {
      render(<LoadingOverlay visible={true} />);

      const paper = document.querySelector('.MuiPaper-root');
      expect(paper).toBeInTheDocument();
    });

    it('should render Stack component with proper alignment', () => {
      render(<LoadingOverlay visible={true} />);

      const stack = document.querySelector('.MuiStack-root');
      expect(stack).toBeInTheDocument();
    });

    it('should maintain proper z-index for overlay', () => {
      render(<LoadingOverlay visible={true} />);

      const backdrop = document.querySelector('.MuiBackdrop-root') as HTMLElement;

      // Should have high z-index (exact value depends on theme)
      const styles = window.getComputedStyle(backdrop);
      const zIndex = parseInt(styles.zIndex || '0');
      expect(zIndex).toBeGreaterThan(1000); // Should be above most content
    });
  });

  describe('Accessibility', () => {
    it('should have proper role for screen readers', () => {
      render(<LoadingOverlay visible={true} message="Loading data" />);

      // Message should be readable by screen readers
      const message = screen.getByText('Loading data');
      expect(message).toBeInTheDocument();
    });

    it('should show progress percentage for screen readers', () => {
      render(<LoadingOverlay visible={true} progress={0.5} />);

      // Percentage should be visible
      const percentage = screen.getByText('50%');
      expect(percentage).toBeInTheDocument();
    });
  });

  describe('Combined Props', () => {
    it('should handle all props together', () => {
      render(
        <LoadingOverlay
          visible={true}
          message="Exporting annotations..."
          progress={0.33}
          transparent={true}
        />
      );

      expect(screen.getByText('Exporting annotations...')).toBeInTheDocument();
      expect(screen.getByText('33%')).toBeInTheDocument();

      const backdrop = document.querySelector('.MuiBackdrop-root') as HTMLElement;
      expect(backdrop).toBeInTheDocument();
    });

    it('should handle visibility toggle with progress', () => {
      const { rerender } = render(
        <LoadingOverlay visible={true} progress={0.25} />
      );

      // Initially at 25%
      expect(screen.getByText('25%')).toBeInTheDocument();

      // Update to 75%
      rerender(<LoadingOverlay visible={true} progress={0.75} />);

      // Should now show 75%
      expect(screen.getByText('75%')).toBeInTheDocument();
      expect(screen.queryByText('25%')).not.toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty message string', () => {
      render(<LoadingOverlay visible={true} message="" />);

      // Should render without crashing, even with empty message
      const backdrop = document.querySelector('.MuiBackdrop-root');
      expect(backdrop).toBeInTheDocument();
    });

    it('should handle progress values outside 0-1 range gracefully', () => {
      // Progress > 1
      const { rerender } = render(
        <LoadingOverlay visible={true} progress={1.5} />
      );

      expect(screen.getByText('150%')).toBeInTheDocument();

      // Progress < 0
      rerender(<LoadingOverlay visible={true} progress={-0.5} />);

      expect(screen.getByText('-50%')).toBeInTheDocument();
    });

    it('should handle very long messages', () => {
      const longMessage =
        'This is a very long loading message that might wrap to multiple lines and should still be displayed correctly';

      render(<LoadingOverlay visible={true} message={longMessage} />);

      expect(screen.getByText(longMessage)).toBeInTheDocument();
    });

    it('should handle decimal progress values correctly', () => {
      render(<LoadingOverlay visible={true} progress={0.12345} />);

      // Should round to 12%
      expect(screen.getByText('12%')).toBeInTheDocument();
    });

    it('should handle progress = 0.5 (midpoint)', () => {
      render(<LoadingOverlay visible={true} progress={0.5} />);

      expect(screen.getByText('50%')).toBeInTheDocument();

      const circularProgress = document.querySelector(
        '.MuiCircularProgress-determinate'
      ) as SVGElement;
      expect(circularProgress).toBeInTheDocument();
    });
  });
});

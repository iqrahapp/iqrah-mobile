/**
 * Component to render Quranic text with tajweed colors and hover tooltips
 */

import React, { useRef, useEffect, useState } from 'react';
import { Box, Popper, Paper, Typography } from '@mui/material';
import { RULE_NAMES, RULE_DESCRIPTIONS } from '../constants/tajweed';

interface TajweedTextProps {
  htmlText: string;
  fontSize?: number;
}

// PERF FIX #3.6: Memoize component to prevent unnecessary re-renders
const TajweedText: React.FC<TajweedTextProps> = React.memo(({ htmlText, fontSize = 24 }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [tooltip, setTooltip] = useState<{
    text: string;
    description: string;
    anchorEl: HTMLElement | null;
  } | null>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleMouseOver = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.tagName.toLowerCase() === 'rule') {
        const ruleClass = target.getAttribute('class');
        if (ruleClass) {
          setTooltip({
            text: RULE_NAMES[ruleClass] || ruleClass,
            description: RULE_DESCRIPTIONS[ruleClass] || '',
            anchorEl: target,
          });
        }
      }
    };

    const handleMouseOut = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.tagName.toLowerCase() === 'rule') {
        setTooltip(null);
      }
    };

    container.addEventListener('mouseover', handleMouseOver);
    container.addEventListener('mouseout', handleMouseOut);

    return () => {
      container.removeEventListener('mouseover', handleMouseOver);
      container.removeEventListener('mouseout', handleMouseOut);
    };
  }, [htmlText]);

  return (
    <>
      <Box
        ref={containerRef}
        component="div"
        sx={{
          fontFamily: '"Amiri Quran", "Traditional Arabic", "Arabic Typesetting", serif',
          fontSize: `${fontSize}px`,
          lineHeight: 2,
          direction: 'rtl',
          textAlign: 'right',
          '& rule': {
            cursor: 'help',
            position: 'relative',
          },
        '& rule[class="ghunnah"]': {
          color: '#4CAF50', // Green
        },
        '& rule[class="qalaqah"]': {
          color: '#FF5722', // Deep Orange
        },
        '& rule[class="madda_normal"]': {
          color: '#2196F3', // Blue
        },
        '& rule[class="madda_permissible"]': {
          color: '#03A9F4', // Light Blue
        },
        '& rule[class="madda_obligatory_mottasel"]': {
          color: '#1976D2', // Dark Blue
        },
        '& rule[class="madda_obligatory_monfasel"]': {
          color: '#0D47A1', // Darker Blue
        },
        '& rule[class="madda_necessary"]': {
          color: '#01579B', // Very Dark Blue
        },
        '& rule[class="ham_wasl"]': {
          color: '#9C27B0', // Purple
        },
        '& rule[class="laam_shamsiyah"]': {
          color: '#F44336', // Red
        },
        '& rule[class="idgham_ghunnah"]': {
          color: '#8BC34A', // Light Green
        },
        '& rule[class="idgham_wo_ghunnah"]': {
          color: '#689F38', // Dark Green
        },
        '& rule[class="idgham_mutajanisayn"]': {
          color: '#558B2F', // Darker Green
        },
        '& rule[class="idgham_shafawi"]': {
          color: '#7CB342', // Yellow Green
        },
        '& rule[class="ikhafa"]': {
          color: '#FFC107', // Amber
        },
        '& rule[class="ikhafa_shafawi"]': {
          color: '#FFB300', // Darker Amber
        },
        '& rule[class="iqlab"]': {
          color: '#FF9800', // Orange
        },
        '& rule[class="slnt"]': {
          color: '#757575', // Grey
        },
      }}
      dangerouslySetInnerHTML={{ __html: htmlText }}
    />

    {tooltip && tooltip.anchorEl && (
      <Popper
        open
        anchorEl={tooltip.anchorEl}
        placement="top"
        sx={{ zIndex: 9999 }}
      >
        <Paper sx={{ p: 1, maxWidth: 250 }}>
          <Typography variant="subtitle2" sx={{ fontWeight: 'bold' }}>
            {tooltip.text}
          </Typography>
          {tooltip.description && (
            <Typography variant="caption" color="text.secondary">
              {tooltip.description}
            </Typography>
          )}
        </Paper>
      </Popper>
    )}
    </>
  );
}, (prevProps, nextProps) =>
  // Custom comparison: only re-render if props actually changed
  prevProps.htmlText === nextProps.htmlText && prevProps.fontSize === nextProps.fontSize
);

export default TajweedText;

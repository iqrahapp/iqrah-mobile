/**
 * Component to render Quranic text with tajweed colors
 */

import React from 'react';
import { Box } from '@mui/material';

interface TajweedTextProps {
  htmlText: string;
  fontSize?: number;
}

const TajweedText: React.FC<TajweedTextProps> = ({ htmlText, fontSize = 24 }) => {
  return (
    <Box
      component="div"
      sx={{
        fontFamily: '"Amiri Quran", "Traditional Arabic", "Arabic Typesetting", serif',
        fontSize: `${fontSize}px`,
        lineHeight: 2,
        direction: 'rtl',
        textAlign: 'right',
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
  );
};

export default TajweedText;

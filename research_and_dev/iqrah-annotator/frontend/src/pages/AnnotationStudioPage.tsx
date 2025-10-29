import React from 'react';
import { Container, Grid } from '@mui/material';
import { FilterAndSelectionPanel } from '../components/studio/FilterAndSelectionPanel';
import { WorkspacePanel } from '../components/studio/WorkspacePanel';
import { DetailPanel } from '../components/studio/DetailPanel';

const AnnotationStudioPage: React.FC = () => {
  return (
    <Container maxWidth="xl" sx={{ py: 3, height: '100vh' }}>
      <Grid container spacing={2} sx={{ height: '100%' }}>
        <Grid item xs={12} md={3} sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          <FilterAndSelectionPanel />
        </Grid>
        <Grid item xs={12} md={6} sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          <WorkspacePanel />
        </Grid>
        <Grid item xs={12} md={3} sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          <DetailPanel />
        </Grid>
      </Grid>
    </Container>
  );
};

export default AnnotationStudioPage;

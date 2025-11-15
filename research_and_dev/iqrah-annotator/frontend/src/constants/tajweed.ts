export const RULE_NAMES: Record<string, string> = {
  ghunnah: 'Ghunnah (Nasal Sound)',
  qalqalah: 'Qalqalah (Echo/Bounce)',
  madda_normal: 'Madd (Normal Lengthening)',
  madda_permissible: 'Madd (Permissible)',
  madda_obligatory_mottasel: 'Madd (Obligatory Connected)',
  madda_obligatory_monfasel: 'Madd (Obligatory Separated)',
  madda_necessary: 'Madd (Necessary)',
  ham_wasl: 'Hamzat al-Wasl (Connecting Hamza)',
  laam_shamsiyah: 'Lam Shamsiyyah (Solar Lam)',
  idgham_ghunnah: 'Idgham with Ghunnah (Merging with Nasal)',
  idgham_wo_ghunnah: 'Idgham without Ghunnah (Merging)',
  idgham_mutajanisayn: 'Idgham Mutajanisayn (Similar Merging)',
  idgham_shafawi: 'Idgham Shafawi (Labial Merging)',
  ikhafa: 'Ikhfa (Hiding)',
  ikhafa_shafawi: 'Ikhfa Shafawi (Labial Hiding)',
  iqlab: 'Iqlab (Conversion)',
  slnt: 'Silent Letter',
};

export const RULE_DESCRIPTIONS: Record<string, string> = {
  ghunnah: 'Nasal sound held for 2 counts',
  qalqalah: 'Echoing/bouncing sound on specific letters',
  madda_normal: 'Elongate for 2 counts',
  madda_permissible: 'Elongate for 2, 4, or 6 counts',
  madda_obligatory_mottasel: 'Elongate for 4-5 counts (connected)',
  madda_obligatory_monfasel: 'Elongate for 4-5 counts (separated)',
  madda_necessary: 'Elongate for 6 counts',
  ham_wasl: 'Connect without pause, drop hamza',
  laam_shamsiyah: 'Lam absorbed into following letter',
  idgham_ghunnah: 'Merge with nasal sound for 2 counts',
  idgham_wo_ghunnah: 'Merge fully without nasal',
  idgham_mutajanisayn: 'Merge similar letters',
  idgham_shafawi: 'Merge using lips',
  ikhafa: 'Hide the noon sound',
  ikhafa_shafawi: 'Hide meem sound with lips',
  iqlab: 'Convert noon to meem',
  slnt: 'Do not pronounce',
};

export const OVERLAP_ALLOWED_RULES = [
  'idgham_ghunnah',
  'idgham_wo_ghunnah',
  'idgham_mutajanisayn',
  'idgham_shafawi',
  'ikhafa',
  'ikhafa_shafawi'
];

// FIX #8: Increased from 100ms to 150ms for merging rules (idgham, ikhfa)
export const MAX_OVERLAP_MS = 150;

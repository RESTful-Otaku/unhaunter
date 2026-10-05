use crate::types::ghost::types::GhostType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GhostSet {
    TmpEMF,
    TmpEMFUVOrbs,
    TmpEMFUVOrbsEVPCPM,
    Twenty,
    #[default]
    All,
}

impl GhostSet {
    pub fn as_vec(&self) -> Vec<GhostType> {
        use GhostType::*;

        match self {
            Self::TmpEMF => vec![LadyInWhite, BrownLady],
            Self::TmpEMFUVOrbs => vec![Caoilte, Ceara, Orla, Finvarra, Kappa, GrayMan],
            Self::TmpEMFUVOrbsEVPCPM => vec![
                Bugbear, Morag, Barghest, Boggart, Obayifo, WillOWisp, LaLlorona, Widow,
                Leprechaun, Brume,
            ],
            Self::Twenty => vec![
                Curupira,
                LaLlorona,
                Phooka,
                Obayifo,
                Maresca,
                Dybbuk,
                Caoilte,
                Orla,
                Jorogumo,
                Mider,
                Aswang,
                Cairbre,
                Ceara,
                Widow,
                BeanSidhe,
                Bugbear,
                Dullahan,
                BaobhanSith,
                Muirgheas,
                Namahage,
            ],
            Self::All => GhostType::all().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::evidence::Evidence;
    use bevy_platform::collections::HashMap;
    use bevy_platform::collections::HashSet;
    use enum_iterator::all;

    #[test]
    fn test_generate_evidence_combinations() {
        // There are exactly C(8,5) = 56 masks with five bits set.
        let mut five_bit_masks = 0;
        for i in 0..256 {
            let nbits = (0..8).filter(|n| (i >> n) & 0x1 > 0).count();
            if nbits == 5 {
                five_bit_masks += 1;
            }
        }
        assert_eq!(five_bit_masks, 56);
    }

    #[test]
    fn test_unique_evidence_combinations() {
        let mut all_combinations: HashSet<String> = HashSet::new();
        for ghost in all::<GhostType>() {
            let mut evidences = ghost
                .evidences()
                .into_iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            evidences.sort();
            let evidences = evidences.join("|");
            assert!(
                all_combinations.insert(evidences),
                "Found duplicate evidence set for {:?}",
                ghost
            );
        }
    }

    #[test]
    fn test_evidence_per_ghost() {
        for ghost in all::<GhostType>() {
            let evidences = ghost
                .evidences()
                .into_iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            assert!(
                evidences.len() == 5,
                "The ghost {:?} does not have 5 evidences, instead it has: {:?}",
                ghost,
                evidences
            );
        }
    }

    #[test]
    fn test_balanced_evidence_usage() {
        let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();
        for ghost in all::<GhostType>() {
            for &evidence in &ghost.evidences() {
                *evidence_count.entry(evidence).or_insert(0) += 1;
            }
        }

        // Assuming a balanced distribution, each evidence should be used roughly the same
        // number of times.
        let avg_use = evidence_count.values().sum::<usize>() / evidence_count.len();
        for (&evidence, &count) in &evidence_count {
            assert!(
                (count as i32 - avg_use as i32).abs() <= 3,
                "Evidence {:?} is used an unbalanced number of times: {} (avg: {})",
                evidence,
                count,
                avg_use
            );
        }
    }
}

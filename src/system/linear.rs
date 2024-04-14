use std::time::Duration;

use async_std::task::sleep;
use derive_getters::Getters;
use linux_embedded_hal::CdevPin;
use serde::{Deserialize, Serialize};

use super::gpio_pin::LocalGpioConfig;

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct LocalLinearConfig {
    pub reverse: bool,
    pub step_pin: LocalGpioConfig,
    pub dir_pin: LocalGpioConfig,
    pub limit: u32,
    pub position: u32,
    pub min_mm_per_s: u32,
    pub max_mm_per_s: u32,
    pub step_per_mm: u32,
}

impl Default for LocalLinearConfig {
    fn default() -> Self {
        Self {
            reverse: false,
            step_pin: Default::default(),
            dir_pin: Default::default(),
            limit: 200,
            position: 0,
            min_mm_per_s: 10,
            max_mm_per_s: 50,
            step_per_mm: 40,
        }
    }
}

impl LocalLinearConfig {
    pub async fn goto(&mut self, position: u32) -> anyhow::Result<u32> {
        let step_pos = position * self.step_per_mm();
        let mut this_step_pos = self.position * self.step_per_mm();

        let max_step_ps = self.max_mm_per_s * self.step_per_mm();
        let min_step_ps = self.min_mm_per_s * self.step_per_mm();

        if (step_pos < this_step_pos) ^ self.reverse() {
            self.dir_pin.set_low()?;
        } else {
            self.dir_pin.set_high()?;
        }

        sleep(Duration::from_millis(1)).await;

        let step = this_step_pos.abs_diff(step_pos);

        while this_step_pos != step_pos {
            let idx = (this_step_pos.abs_diff(step_pos) * 999) / step;

            let v = SINE_TO_PI[idx as usize] * (max_step_ps - min_step_ps) + min_step_ps * 1000;

            self.step_pin.set_high()?;
            sleep(Duration::from_micros(2)).await;
            self.step_pin.set_low()?;
            sleep(Duration::from_micros(1_000_000_000 / v as u64)).await;
            if this_step_pos < step_pos {
                this_step_pos += 1;
            } else {
                this_step_pos -= 1;
            }
        }

        self.position = position;
        Ok(position)
    }
}

pub struct StepperLinear {
    dir_pin: Option<CdevPin>,
    step_pin: Option<CdevPin>,

    pub dir_pin_conf: LocalGpioConfig,
    pub step_pin_conf: LocalGpioConfig,
    pub reversed: bool,
    pub limit: u32,
    pub step_pos: u32,
    pub min_step_ps: u32,
    pub max_step_ps: u32,
    pub step_p_mm: u32,
}

impl From<&LocalLinearConfig> for StepperLinear {
    fn from(value: &LocalLinearConfig) -> Self {
        Self::from(value.clone())
    }
}

impl From<LocalLinearConfig> for StepperLinear {
    fn from(value: LocalLinearConfig) -> Self {
        Self {
            dir_pin: None,
            step_pin: None,

            dir_pin_conf: value.dir_pin,
            step_pin_conf: value.step_pin,
            reversed: value.reverse,
            limit: value.limit,
            step_pos: 0,
            min_step_ps: value.min_mm_per_s * value.step_per_mm,
            max_step_ps: value.max_mm_per_s * value.step_per_mm,
            step_p_mm: value.step_per_mm,
        }
    }
}

//impl StepperLinear {
//    //pub fn new(settings: &LocalLinearConfig) -> anyhow::Result<Self> {
//    //    let dir_pin = Some(get_output(settings.dir_pin())?);
//    //    let step_pin = Some(get_output(settings.step_pin())?);
//    //    Ok(Self {
//    //        dir_pin,
//    //        step_pin,
//    //        //en_pin,
//    //        //diag_pin,
//    //        reversed: *settings.reverse(),
//    //        //lower_limit: *settings.lower_limit(),
//    //        //upper_limit: *settings.upper_limit(),
//    //        step_pos: 0,
//    //        min_step_ps: *settings.min_mm_per_s() * settings.step_per_mm(),
//    //        max_step_ps: *settings.max_mm_per_s() * settings.step_per_mm(),
//    //        step_p_mm: *settings.step_per_mm(),
//    //    })
//    //}
//
//    pub async fn goto(&mut self, position: u32) -> anyhow::Result<u32> {
//        let mut dir_pin = loop {
//            if let Some(pin) = self.dir_pin.take() {
//                break pin;
//            } else {
//                self.dir_pin.replace(get_output(&self.dir_pin_conf)?);
//            };
//        };
//
//        let mut step_pin = loop {
//            if let Some(pin) = self.step_pin.take() {
//                break pin;
//            } else {
//                self.step_pin.replace(get_output(&self.step_pin_conf)?);
//            };
//        };
//
//        let step_pos = position * self.step_p_mm;
//
//        if (step_pos < self.step_pos) ^ self.reversed {
//            dir_pin.set_low()?;
//
//            sleep(Duration::from_millis(1)).await;
//
//            let step = self.step_pos.abs_diff(step_pos);
//
//            while self.step_pos > step_pos {
//                let idx = (self.step_pos.abs_diff(step_pos) * 999) / step;
//
//                let v = Self::SINE_TO_PI[idx as usize] * (self.max_step_ps - self.min_step_ps)
//                    + self.min_step_ps;
//
//                step_pin.set_high()?;
//                sleep(Duration::from_micros(2)).await;
//                step_pin.set_low()?;
//                sleep(Duration::from_micros(1_000_000_000 / v as u64)).await;
//                self.step_pos -= 1;
//            }
//        } else {
//            dir_pin.set_high()?;
//
//            sleep(Duration::from_millis(1)).await;
//
//            let step = self.step_pos.abs_diff(step_pos);
//
//            while self.step_pos < step_pos {
//                let idx = (self.step_pos.abs_diff(step_pos) * 999) / step;
//
//                let v = Self::SINE_TO_PI[idx as usize] * (self.max_step_ps - self.min_step_ps)
//                    + self.min_step_ps;
//
//                step_pin.set_high()?;
//                sleep(Duration::from_micros(2)).await;
//                step_pin.set_low()?;
//                sleep(Duration::from_micros(1_000_000_000 / v as u64)).await;
//                self.step_pos += 1;
//            }
//        }
//
//        self.dir_pin.replace(dir_pin);
//        self.step_pin.replace(step_pin);
//        Ok(position)
//    }
//    const SINE_TO_PI: [u32; 1000] = [
//        0, 3, 6, 9, 12, 15, 18, 21, 25, 28, 31, 34, 37, 40, 43, 47, 50, 53, 56, 59, 62, 65, 69, 72,
//        75, 78, 81, 84, 87, 90, 94, 97, 100, 103, 106, 109, 112, 115, 119, 122, 125, 128, 131, 134,
//        137, 140, 144, 147, 150, 153, 156, 159, 162, 165, 168, 171, 175, 178, 181, 184, 187, 190,
//        193, 196, 199, 202, 205, 208, 212, 215, 218, 221, 224, 227, 230, 233, 236, 239, 242, 245,
//        248, 251, 254, 257, 260, 263, 266, 269, 272, 275, 278, 282, 285, 288, 291, 294, 297, 300,
//        303, 306, 309, 312, 314, 317, 320, 323, 326, 329, 332, 335, 338, 341, 344, 347, 350, 353,
//        356, 359, 362, 365, 368, 371, 373, 376, 379, 382, 385, 388, 391, 394, 397, 400, 402, 405,
//        408, 411, 414, 417, 420, 422, 425, 428, 431, 434, 437, 439, 442, 445, 448, 451, 453, 456,
//        459, 462, 465, 467, 470, 473, 476, 478, 481, 484, 487, 489, 492, 495, 498, 500, 503, 506,
//        509, 511, 514, 517, 519, 522, 525, 527, 530, 533, 535, 538, 541, 543, 546, 549, 551, 554,
//        556, 559, 562, 564, 567, 569, 572, 575, 577, 580, 582, 585, 587, 590, 592, 595, 597, 600,
//        602, 605, 607, 610, 612, 615, 617, 620, 622, 625, 627, 630, 632, 635, 637, 639, 642, 644,
//        647, 649, 651, 654, 656, 658, 661, 663, 666, 668, 670, 673, 675, 677, 679, 682, 684, 686,
//        689, 691, 693, 695, 698, 700, 702, 704, 707, 709, 711, 713, 715, 718, 720, 722, 724, 726,
//        728, 731, 733, 735, 737, 739, 741, 743, 745, 748, 750, 752, 754, 756, 758, 760, 762, 764,
//        766, 768, 770, 772, 774, 776, 778, 780, 782, 784, 786, 788, 790, 792, 793, 795, 797, 799,
//        801, 803, 805, 807, 809, 810, 812, 814, 816, 818, 819, 821, 823, 825, 827, 828, 830, 832,
//        834, 835, 837, 839, 840, 842, 844, 846, 847, 849, 850, 852, 854, 855, 857, 859, 860, 862,
//        863, 865, 867, 868, 870, 871, 873, 874, 876, 877, 879, 880, 882, 883, 885, 886, 888, 889,
//        891, 892, 893, 895, 896, 898, 899, 900, 902, 903, 904, 906, 907, 908, 910, 911, 912, 913,
//        915, 916, 917, 918, 920, 921, 922, 923, 925, 926, 927, 928, 929, 930, 932, 933, 934, 935,
//        936, 937, 938, 939, 940, 941, 942, 944, 945, 946, 947, 948, 949, 950, 951, 952, 952, 953,
//        954, 955, 956, 957, 958, 959, 960, 961, 962, 962, 963, 964, 965, 966, 967, 967, 968, 969,
//        970, 970, 971, 972, 973, 973, 974, 975, 975, 976, 977, 977, 978, 979, 979, 980, 981, 981,
//        982, 982, 983, 984, 984, 985, 985, 986, 986, 987, 987, 988, 988, 989, 989, 990, 990, 990,
//        991, 991, 992, 992, 992, 993, 993, 993, 994, 994, 994, 995, 995, 995, 996, 996, 996, 996,
//        997, 997, 997, 997, 998, 998, 998, 998, 998, 998, 999, 999, 999, 999, 999, 999, 999, 999,
//        999, 999, 999, 999, 999, 999, 1000, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999,
//        999, 999, 999, 998, 998, 998, 998, 998, 998, 997, 997, 997, 997, 996, 996, 996, 996, 995,
//        995, 995, 994, 994, 994, 993, 993, 993, 992, 992, 992, 991, 991, 990, 990, 990, 989, 989,
//        988, 988, 987, 987, 986, 986, 985, 985, 984, 984, 983, 982, 982, 981, 981, 980, 979, 979,
//        978, 977, 977, 976, 975, 975, 974, 973, 973, 972, 971, 970, 970, 969, 968, 967, 967, 966,
//        965, 964, 963, 962, 962, 961, 960, 959, 958, 957, 956, 955, 954, 953, 952, 952, 951, 950,
//        949, 948, 947, 946, 945, 944, 942, 941, 940, 939, 938, 937, 936, 935, 934, 933, 932, 930,
//        929, 928, 927, 926, 925, 923, 922, 921, 920, 918, 917, 916, 915, 913, 912, 911, 910, 908,
//        907, 906, 904, 903, 902, 900, 899, 898, 896, 895, 893, 892, 891, 889, 888, 886, 885, 883,
//        882, 880, 879, 877, 876, 874, 873, 871, 870, 868, 867, 865, 863, 862, 860, 859, 857, 855,
//        854, 852, 850, 849, 847, 846, 844, 842, 840, 839, 837, 835, 834, 832, 830, 828, 827, 825,
//        823, 821, 819, 818, 816, 814, 812, 810, 809, 807, 805, 803, 801, 799, 797, 795, 793, 792,
//        790, 788, 786, 784, 782, 780, 778, 776, 774, 772, 770, 768, 766, 764, 762, 760, 758, 756,
//        754, 752, 750, 748, 745, 743, 741, 739, 737, 735, 733, 731, 728, 726, 724, 722, 720, 718,
//        715, 713, 711, 709, 707, 704, 702, 700, 698, 695, 693, 691, 689, 686, 684, 682, 679, 677,
//        675, 673, 670, 668, 666, 663, 661, 658, 656, 654, 651, 649, 647, 644, 642, 639, 637, 635,
//        632, 630, 627, 625, 622, 620, 617, 615, 612, 610, 607, 605, 602, 600, 597, 595, 592, 590,
//        587, 585, 582, 580, 577, 575, 572, 569, 567, 564, 562, 559, 556, 554, 551, 549, 546, 543,
//        541, 538, 535, 533, 530, 527, 525, 522, 519, 517, 514, 511, 509, 506, 503, 500, 498, 495,
//        492, 489, 487, 484, 481, 478, 476, 473, 470, 467, 465, 462, 459, 456, 453, 451, 448, 445,
//        442, 439, 437, 434, 431, 428, 425, 422, 420, 417, 414, 411, 408, 405, 402, 400, 397, 394,
//        391, 388, 385, 382, 379, 376, 373, 371, 368, 365, 362, 359, 356, 353, 350, 347, 344, 341,
//        338, 335, 332, 329, 326, 323, 320, 317, 314, 312, 309, 306, 303, 300, 297, 294, 291, 288,
//        285, 282, 278, 275, 272, 269, 266, 263, 260, 257, 254, 251, 248, 245, 242, 239, 236, 233,
//        230, 227, 224, 221, 218, 215, 212, 208, 205, 202, 199, 196, 193, 190, 187, 184, 181, 178,
//        175, 171, 168, 165, 162, 159, 156, 153, 150, 147, 144, 140, 137, 134, 131, 128, 125, 122,
//        119, 115, 112, 109, 106, 103, 100, 97, 94, 90, 87, 84, 81, 78, 75, 72, 69, 65, 62, 59, 56,
//        53, 50, 47, 43, 40, 37, 34, 31, 28, 25, 21, 18, 15, 12, 9, 6, 3,
//    ];
//}
//
const SINE_TO_PI: [u32; 1000] = [
    0, 3, 6, 9, 12, 15, 18, 21, 25, 28, 31, 34, 37, 40, 43, 47, 50, 53, 56, 59, 62, 65, 69, 72, 75,
    78, 81, 84, 87, 90, 94, 97, 100, 103, 106, 109, 112, 115, 119, 122, 125, 128, 131, 134, 137,
    140, 144, 147, 150, 153, 156, 159, 162, 165, 168, 171, 175, 178, 181, 184, 187, 190, 193, 196,
    199, 202, 205, 208, 212, 215, 218, 221, 224, 227, 230, 233, 236, 239, 242, 245, 248, 251, 254,
    257, 260, 263, 266, 269, 272, 275, 278, 282, 285, 288, 291, 294, 297, 300, 303, 306, 309, 312,
    314, 317, 320, 323, 326, 329, 332, 335, 338, 341, 344, 347, 350, 353, 356, 359, 362, 365, 368,
    371, 373, 376, 379, 382, 385, 388, 391, 394, 397, 400, 402, 405, 408, 411, 414, 417, 420, 422,
    425, 428, 431, 434, 437, 439, 442, 445, 448, 451, 453, 456, 459, 462, 465, 467, 470, 473, 476,
    478, 481, 484, 487, 489, 492, 495, 498, 500, 503, 506, 509, 511, 514, 517, 519, 522, 525, 527,
    530, 533, 535, 538, 541, 543, 546, 549, 551, 554, 556, 559, 562, 564, 567, 569, 572, 575, 577,
    580, 582, 585, 587, 590, 592, 595, 597, 600, 602, 605, 607, 610, 612, 615, 617, 620, 622, 625,
    627, 630, 632, 635, 637, 639, 642, 644, 647, 649, 651, 654, 656, 658, 661, 663, 666, 668, 670,
    673, 675, 677, 679, 682, 684, 686, 689, 691, 693, 695, 698, 700, 702, 704, 707, 709, 711, 713,
    715, 718, 720, 722, 724, 726, 728, 731, 733, 735, 737, 739, 741, 743, 745, 748, 750, 752, 754,
    756, 758, 760, 762, 764, 766, 768, 770, 772, 774, 776, 778, 780, 782, 784, 786, 788, 790, 792,
    793, 795, 797, 799, 801, 803, 805, 807, 809, 810, 812, 814, 816, 818, 819, 821, 823, 825, 827,
    828, 830, 832, 834, 835, 837, 839, 840, 842, 844, 846, 847, 849, 850, 852, 854, 855, 857, 859,
    860, 862, 863, 865, 867, 868, 870, 871, 873, 874, 876, 877, 879, 880, 882, 883, 885, 886, 888,
    889, 891, 892, 893, 895, 896, 898, 899, 900, 902, 903, 904, 906, 907, 908, 910, 911, 912, 913,
    915, 916, 917, 918, 920, 921, 922, 923, 925, 926, 927, 928, 929, 930, 932, 933, 934, 935, 936,
    937, 938, 939, 940, 941, 942, 944, 945, 946, 947, 948, 949, 950, 951, 952, 952, 953, 954, 955,
    956, 957, 958, 959, 960, 961, 962, 962, 963, 964, 965, 966, 967, 967, 968, 969, 970, 970, 971,
    972, 973, 973, 974, 975, 975, 976, 977, 977, 978, 979, 979, 980, 981, 981, 982, 982, 983, 984,
    984, 985, 985, 986, 986, 987, 987, 988, 988, 989, 989, 990, 990, 990, 991, 991, 992, 992, 992,
    993, 993, 993, 994, 994, 994, 995, 995, 995, 996, 996, 996, 996, 997, 997, 997, 997, 998, 998,
    998, 998, 998, 998, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 1000,
    999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 999, 998, 998, 998, 998, 998,
    998, 997, 997, 997, 997, 996, 996, 996, 996, 995, 995, 995, 994, 994, 994, 993, 993, 993, 992,
    992, 992, 991, 991, 990, 990, 990, 989, 989, 988, 988, 987, 987, 986, 986, 985, 985, 984, 984,
    983, 982, 982, 981, 981, 980, 979, 979, 978, 977, 977, 976, 975, 975, 974, 973, 973, 972, 971,
    970, 970, 969, 968, 967, 967, 966, 965, 964, 963, 962, 962, 961, 960, 959, 958, 957, 956, 955,
    954, 953, 952, 952, 951, 950, 949, 948, 947, 946, 945, 944, 942, 941, 940, 939, 938, 937, 936,
    935, 934, 933, 932, 930, 929, 928, 927, 926, 925, 923, 922, 921, 920, 918, 917, 916, 915, 913,
    912, 911, 910, 908, 907, 906, 904, 903, 902, 900, 899, 898, 896, 895, 893, 892, 891, 889, 888,
    886, 885, 883, 882, 880, 879, 877, 876, 874, 873, 871, 870, 868, 867, 865, 863, 862, 860, 859,
    857, 855, 854, 852, 850, 849, 847, 846, 844, 842, 840, 839, 837, 835, 834, 832, 830, 828, 827,
    825, 823, 821, 819, 818, 816, 814, 812, 810, 809, 807, 805, 803, 801, 799, 797, 795, 793, 792,
    790, 788, 786, 784, 782, 780, 778, 776, 774, 772, 770, 768, 766, 764, 762, 760, 758, 756, 754,
    752, 750, 748, 745, 743, 741, 739, 737, 735, 733, 731, 728, 726, 724, 722, 720, 718, 715, 713,
    711, 709, 707, 704, 702, 700, 698, 695, 693, 691, 689, 686, 684, 682, 679, 677, 675, 673, 670,
    668, 666, 663, 661, 658, 656, 654, 651, 649, 647, 644, 642, 639, 637, 635, 632, 630, 627, 625,
    622, 620, 617, 615, 612, 610, 607, 605, 602, 600, 597, 595, 592, 590, 587, 585, 582, 580, 577,
    575, 572, 569, 567, 564, 562, 559, 556, 554, 551, 549, 546, 543, 541, 538, 535, 533, 530, 527,
    525, 522, 519, 517, 514, 511, 509, 506, 503, 500, 498, 495, 492, 489, 487, 484, 481, 478, 476,
    473, 470, 467, 465, 462, 459, 456, 453, 451, 448, 445, 442, 439, 437, 434, 431, 428, 425, 422,
    420, 417, 414, 411, 408, 405, 402, 400, 397, 394, 391, 388, 385, 382, 379, 376, 373, 371, 368,
    365, 362, 359, 356, 353, 350, 347, 344, 341, 338, 335, 332, 329, 326, 323, 320, 317, 314, 312,
    309, 306, 303, 300, 297, 294, 291, 288, 285, 282, 278, 275, 272, 269, 266, 263, 260, 257, 254,
    251, 248, 245, 242, 239, 236, 233, 230, 227, 224, 221, 218, 215, 212, 208, 205, 202, 199, 196,
    193, 190, 187, 184, 181, 178, 175, 171, 168, 165, 162, 159, 156, 153, 150, 147, 144, 140, 137,
    134, 131, 128, 125, 122, 119, 115, 112, 109, 106, 103, 100, 97, 94, 90, 87, 84, 81, 78, 75, 72,
    69, 65, 62, 59, 56, 53, 50, 47, 43, 40, 37, 34, 31, 28, 25, 21, 18, 15, 12, 9, 6, 3,
];

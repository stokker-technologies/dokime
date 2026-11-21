# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Declare the Shared Thermal Load Streams"""

# === Imports === #

# Standard Library Imports
from collections.abc import Mapping

# isort: split

# Third Party Imports
from dokime_sdk import Scalar
from dokime_sdk.templates import Context, Materialization, Parameter, TestTemplate, Window, materialize, step, stream, window_observer
from google.protobuf.message import Message

# isort: split

# Local Imports
from thermal_example import sampling
from thermal_example.models import Counter, CounterColumns

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ThermalLoadTestTemplate(TestTemplate):
    identifier = "load"
    label = "Thermal Load"
    parameters = [Parameter.flag("burst", "Burst").default(False).build()]

    channel_000 = sampling.channel("channel_000")
    channel_001 = sampling.channel("channel_001")
    channel_002 = sampling.channel("channel_002")
    channel_003 = sampling.channel("channel_003")
    channel_004 = sampling.channel("channel_004")
    channel_005 = sampling.channel("channel_005")
    channel_006 = sampling.channel("channel_006")
    channel_007 = sampling.channel("channel_007")
    channel_008 = sampling.channel("channel_008")
    channel_009 = sampling.channel("channel_009")
    channel_010 = sampling.channel("channel_010")
    channel_011 = sampling.channel("channel_011")
    channel_012 = sampling.channel("channel_012")
    channel_013 = sampling.channel("channel_013")
    channel_014 = sampling.channel("channel_014")
    channel_015 = sampling.channel("channel_015")
    channel_016 = sampling.channel("channel_016")
    channel_017 = sampling.channel("channel_017")
    channel_018 = sampling.channel("channel_018")
    channel_019 = sampling.channel("channel_019")
    channel_020 = sampling.channel("channel_020")
    channel_021 = sampling.channel("channel_021")
    channel_022 = sampling.channel("channel_022")
    channel_023 = sampling.channel("channel_023")
    channel_024 = sampling.channel("channel_024")
    channel_025 = sampling.channel("channel_025")
    channel_026 = sampling.channel("channel_026")
    channel_027 = sampling.channel("channel_027")
    channel_028 = sampling.channel("channel_028")
    channel_029 = sampling.channel("channel_029")
    channel_030 = sampling.channel("channel_030")
    channel_031 = sampling.channel("channel_031")
    channel_032 = sampling.channel("channel_032")
    channel_033 = sampling.channel("channel_033")
    channel_034 = sampling.channel("channel_034")
    channel_035 = sampling.channel("channel_035")
    channel_036 = sampling.channel("channel_036")
    channel_037 = sampling.channel("channel_037")
    channel_038 = sampling.channel("channel_038")
    channel_039 = sampling.channel("channel_039")
    channel_040 = sampling.channel("channel_040")
    channel_041 = sampling.channel("channel_041")
    channel_042 = sampling.channel("channel_042")
    channel_043 = sampling.channel("channel_043")
    channel_044 = sampling.channel("channel_044")
    channel_045 = sampling.channel("channel_045")
    channel_046 = sampling.channel("channel_046")
    channel_047 = sampling.channel("channel_047")
    channel_048 = sampling.channel("channel_048")
    channel_049 = sampling.channel("channel_049")
    channel_050 = sampling.channel("channel_050")
    channel_051 = sampling.channel("channel_051")
    channel_052 = sampling.channel("channel_052")
    channel_053 = sampling.channel("channel_053")
    channel_054 = sampling.channel("channel_054")
    channel_055 = sampling.channel("channel_055")
    channel_056 = sampling.channel("channel_056")
    channel_057 = sampling.channel("channel_057")
    channel_058 = sampling.channel("channel_058")
    channel_059 = sampling.channel("channel_059")
    channel_060 = sampling.channel("channel_060")
    channel_061 = sampling.channel("channel_061")
    channel_062 = sampling.channel("channel_062")
    channel_063 = sampling.channel("channel_063")
    channel_064 = sampling.channel("channel_064")
    channel_065 = sampling.channel("channel_065")
    channel_066 = sampling.channel("channel_066")
    channel_067 = sampling.channel("channel_067")
    channel_068 = sampling.channel("channel_068")
    channel_069 = sampling.channel("channel_069")
    channel_070 = sampling.channel("channel_070")
    channel_071 = sampling.channel("channel_071")
    channel_072 = sampling.channel("channel_072")
    channel_073 = sampling.channel("channel_073")
    channel_074 = sampling.channel("channel_074")
    channel_075 = sampling.channel("channel_075")
    channel_076 = sampling.channel("channel_076")
    channel_077 = sampling.channel("channel_077")
    channel_078 = sampling.channel("channel_078")
    channel_079 = sampling.channel("channel_079")
    channel_080 = sampling.channel("channel_080")
    channel_081 = sampling.channel("channel_081")
    channel_082 = sampling.channel("channel_082")
    channel_083 = sampling.channel("channel_083")
    channel_084 = sampling.channel("channel_084")
    channel_085 = sampling.channel("channel_085")
    channel_086 = sampling.channel("channel_086")
    channel_087 = sampling.channel("channel_087")
    channel_088 = sampling.channel("channel_088")
    channel_089 = sampling.channel("channel_089")
    channel_090 = sampling.channel("channel_090")
    channel_091 = sampling.channel("channel_091")
    channel_092 = sampling.channel("channel_092")
    channel_093 = sampling.channel("channel_093")
    channel_094 = sampling.channel("channel_094")
    channel_095 = sampling.channel("channel_095")
    channel_096 = sampling.channel("channel_096")
    channel_097 = sampling.channel("channel_097")
    channel_098 = sampling.channel("channel_098")
    channel_099 = sampling.channel("channel_099")
    channel_100 = sampling.channel("channel_100")
    channel_101 = sampling.channel("channel_101")
    channel_102 = sampling.channel("channel_102")
    channel_103 = sampling.channel("channel_103")
    channel_104 = sampling.channel("channel_104")
    channel_105 = sampling.channel("channel_105")
    channel_106 = sampling.channel("channel_106")
    channel_107 = sampling.channel("channel_107")
    channel_108 = sampling.channel("channel_108")
    channel_109 = sampling.channel("channel_109")
    channel_110 = sampling.channel("channel_110")
    channel_111 = sampling.channel("channel_111")
    channel_112 = sampling.channel("channel_112")
    channel_113 = sampling.channel("channel_113")
    channel_114 = sampling.channel("channel_114")
    channel_115 = sampling.channel("channel_115")
    channel_116 = sampling.channel("channel_116")
    channel_117 = sampling.channel("channel_117")
    channel_118 = sampling.channel("channel_118")
    channel_119 = sampling.channel("channel_119")
    channel_120 = sampling.channel("channel_120")
    channel_121 = sampling.channel("channel_121")
    channel_122 = sampling.channel("channel_122")
    channel_123 = sampling.channel("channel_123")
    channel_124 = sampling.channel("channel_124")
    channel_125 = sampling.channel("channel_125")
    channel_126 = sampling.channel("channel_126")
    channel_127 = sampling.channel("channel_127")

    @stream("Single-Stream Ceiling", row=Counter)
    @window_observer(columns=CounterColumns, interval_ms=1, interval_setting="thermal.load.interval")
    def burst(self, ctx: Context, window: Window) -> Message:
        return sampling.counter(window)

    async def initialize(self, ctx: Context) -> None:
        await ctx.progress(0, 1, label="Hold")

    @materialize
    def select(self, case: Materialization, parameters: Mapping[str, Scalar]) -> None:
        burst = parameters["burst"]
        retained = [stream for stream in case.message.streams if stream.endswith(".burst") == burst]
        del case.message.streams[:]
        case.message.streams.extend(retained)
        case.step("Hold", self.hold)

    @step("Hold", identifier="hold")
    async def hold(self, ctx: Context) -> None:
        await ctx.cancelled()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

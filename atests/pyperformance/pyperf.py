# This is a mock of the pyperf module.

def perf_counter():
    return 42 # TODO put something better here?

class Args:
    pass

class ArgParser:
    def __init__(self):
        self.args = Args()

    def add_argument(self, name, default=None, **kwargs):
        if name.startswith("--"): name = name[2:]
        name = name.replace("-", "_")
        self.args.__dict__[name] = default

class Runner:
    def __init__(self, add_cmdline_args=None):
        self.metadata = {}
        self.argparser = ArgParser()

    def parse_args(self):
        return self.argparser.args

    def bench_time_func(self, name, f, *args, **kwargs):
        f(42, *args)

    def bench_func(self, name, f, *args):
        f(*args)

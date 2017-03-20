#!/usr/bin/python3

# Tool that generates comparison data from the core log and the bitcrust log

import tailer
import pprint
import re
import pystache

from dateutil.parser import parse

CORE_LOG = "/home/tomas/.bitcoin/debug.log"
BC_LOG   = "/home/tomas/src/bitcrust/cmp1"

RE_CORE_CONNECT   = re.compile(r'- Connect block: (?P<blocktime>[0-9.]*)ms')
RE_CORE_UPDATETIP = re.compile(r"UpdateTip: new best=(?P<hash>[0-9a-f]*) height=(?P<height>[0-9]*)")
RE_CORE_TXCOUNT   = re.compile(r"Connect (?P<txcount>[0-9]*) transactions")
RE_CORE_TXINCOUNT = re.compile(r"Verify (?P<txincount>[0-9]*) txins")

RE_BC_START     = re.compile(r'^(?P<bc_start>.+) INFO add_block - start')
RE_BC_HASH      = re.compile(r'add_block - hashed, hash: (?P<hash>[0-9a-f]*)') 
RE_BC_END       = re.compile(r'^(?P<bc_done>.+) INFO add_block - done')
RE_BC_START_BE  = re.compile(r'^(?P<bc_connect_start>.+) INFO add_block - block-index')
RE_BC_END_BE    = re.compile(r'^(?P<bc_connect_end>.+) INFO connected')

# reads core  
def read_core():
    print("Reading core");

    log = reversed(tailer.tail(open(CORE_LOG), 1000))

    return log

# reads bc logfile  
def read_bc():
    print("Reading bitcrust");

    log = reversed(tailer.tail(open(BC_LOG), 1000))

    return log

def add_regex_result(obj, regex, line):

    result = regex.search(line)
    if result:
        obj.update(result.groupdict())

# yields blocks
def parse_core_log(log):
    
    block = {}
    for n in log:
        add_regex_result(block, RE_CORE_CONNECT, n)
        add_regex_result(block, RE_CORE_UPDATETIP, n)
        add_regex_result(block, RE_CORE_TXCOUNT, n)
        add_regex_result(block, RE_CORE_TXINCOUNT, n)

        if "txcount" in block:
            yield block

            block = {}

# yields blocks
def parse_bc_log(log):
    
    block = {}
    for n in log:
        if "already exists" in n:
            break
        add_regex_result(block, RE_BC_START, n)
        add_regex_result(block, RE_BC_END, n)
        add_regex_result(block, RE_BC_START_BE, n)
        add_regex_result(block, RE_BC_END_BE, n)
        add_regex_result(block, RE_BC_HASH, n)


        if "bc_start" in block:
            dur = parse(block['bc_done']) - parse(block['bc_start'])
            block['bc_dur'] = dur.total_seconds()*1000
            cdur = parse(block['bc_connect_end']) - parse(block['bc_connect_start'])
            block['bc_cdur'] = cdur.total_seconds()*1000
            del block['bc_done']
            del block['bc_start']
            del block['bc_connect_start']
            del block['bc_connect_end']

            yield block

            block = {}



bc_blocks = list(parse_bc_log(read_bc()))


for block in parse_core_log(read_core()):
    #print "processing", block
    for bc in bc_blocks:
        #print " with ", bc
        if bc['hash'] == block['hash']:
            block.update(bc)
            print block

print pystache.render('Hi {{person}}!', {'person': 'Jad'})


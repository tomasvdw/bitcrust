#!/usr/bin/python3

# Tool that generates comparison data from the core log and the bitcrust log

import tailer
import pprint
import re
import pystache

from dateutil.parser import parse

CORE_LOG = "/home/tomas/.bitcoin/debug.log"
BC_LOG   = "/home/tomas/bitcrust/cmp1"

RE_CORE_CONNECT   = re.compile(r'^(?P<time>.*) - Connect block: (?P<blocktime>[0-9.]*)ms')
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

    log = reversed(tailer.tail(open(CORE_LOG), 1000))

    return log

# reads bc logfile  
def read_bc():

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
            block['blocktime'] = int(float(block['blocktime']))
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
            block['bc_cdur'] = int(float(block['bc_cdur']))
            block['bc_dur'] = int(float(block['bc_dur']))
            del block['bc_done']
            del block['bc_start']
            del block['bc_connect_start']
            del block['bc_connect_end']

            yield block

            block = {}

def render_block(block):
    return pystache.render("""
    <li>
        <dl>
            <dt class="time">time
            <dd class="time">{{time}}

            <dt class="hash">hash
            <dd class="hash">{{hash}}

            <dt class="transactions">transactions
            <dd class="transactions">{{txcount}}

            <dt class="inputs">transactions
            <dd class="inputs">{{txincount}}
        
        
        </dl>
        <table>
            <tr>
                <td style="border-color:green; >{{blocktime}} ms
            <tr>
                <td style="border-color:green; >{{bc_dur}} ms
            <tr>
                <td style="border-color:green; >{{bc_cdur}} ms
        </table>
    </li>
    """, block)

bc_blocks = list(parse_bc_log(read_bc()))


print "<ul class='graph'>"
for block in parse_core_log(read_core()):
    #print "processing", block
    for bc in bc_blocks:
        #print " with ", bc
        if bc['hash'] == block['hash']:
            block.update(bc)
            print render_block(block)


print "</ul>"


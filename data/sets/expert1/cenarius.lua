local card={api_version=1,id="EX1_573",name="Cenarius",text="<b>Choose One -</b> Give your other minions +2/+2; or Summon two 2/2 Treants with <b>Taunt</b>.",set="EXPERT1",type="minion",class="druid",rarity="legendary",cost=8,attack=5,health=8,keywords={"choose_one"}}
local function favor(ctx,self)for _,e in ipairs(ctx:friendly_minions(self))do if e~=self then cardlib.effects.buff(ctx, e,2,2)end end end
local function treants(ctx,self)local p=ctx:controller(self);ctx:summon(p,"EX1_573t");ctx:summon(p,"EX1_573t")end
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{card_id="EX1_573a",label="Give your other minions +2/+2"},{card_id="EX1_573b",label="Summon two 2/2 Treants with Taunt"}},"chosen")end
function card.chosen(ctx,self,c)if c=="EX1_573a" then favor(ctx,self)else treants(ctx,self)end end
function card.on_choose_multiple(ctx,self)favor(ctx,self);treants(ctx,self)end
card.tokens={{id="EX1_573a",name="Demigod's Favor",text="Give your other minions +2/+2.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=8},{id="EX1_573b",name="Shan'do's Lesson",text="Summon two 2/2 Treants with <b>Taunt</b>.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=8},{id="EX1_573t",name="Treant",text="<b>Taunt</b>",set="EXPERT1",type="minion",class="druid",collectible=false,cost=1,attack=2,health=2,keywords={"taunt"}}}
return card

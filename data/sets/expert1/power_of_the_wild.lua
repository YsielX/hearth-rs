local card={api_version=1,id="EX1_160",name="Power of the Wild",text="<b>Choose One -</b> Give your minions +1/+1; or Summon a 3/2 Panther.",set="EXPERT1",type="spell",class="druid",rarity="common",cost=2,keywords={"choose_one"}}
local function buff(ctx,self)for _,e in ipairs(ctx:friendly_minions(self))do cardlib.effects.buff(ctx, e,1,1)end end
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{card_id="EX1_160b",label="Give your minions +1/+1"},{card_id="EX1_160a",label="Summon a 3/2 Panther"}},"chosen")end
function card.chosen(ctx,self,c)if c=="EX1_160b" then buff(ctx,self)else ctx:summon(ctx:controller(self),"EX1_160t")end end
function card.on_choose_multiple(ctx,self)buff(ctx,self);ctx:summon(ctx:controller(self),"EX1_160t")end
card.tokens={{id="EX1_160a",name="Summon a Panther",text="Summon a 3/2 Panther.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=2},{id="EX1_160b",name="Leader of the Pack",text="Give your minions +1/+1.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=2},{id="EX1_160t",name="Panther",text="",set="EXPERT1",type="minion",class="druid",collectible=false,cost=2,attack=3,health=2,tags={"beast"}}}
return card

local card={api_version=1,id="EX1_164",name="Nourish",text="<b>Choose One -</b> Gain 2 Mana Crystals; or Draw 3 cards.",set="EXPERT1",type="spell",class="druid",rarity="rare",spell_school="nature",cost=5,keywords={"choose_one"}}
local function grow(ctx,self)ctx:gain_mana_crystals(ctx:controller(self),2,false)end
local function draw(ctx,self)ctx:draw(ctx:controller(self),3)end
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{card_id="EX1_164a",label="Gain 2 Mana Crystals"},{card_id="EX1_164b",label="Draw 3 cards"}},"chosen")end
function card.chosen(ctx,self,c)if c=="EX1_164a" then grow(ctx,self)else draw(ctx,self)end end
function card.on_choose_multiple(ctx,self)grow(ctx,self);draw(ctx,self)end
card.tokens={{id="EX1_164a", spell_school = "nature",name="Rampant Growth",text="Gain 2 Mana Crystals.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=5},{id="EX1_164b", spell_school = "nature",name="Enrich",text="Draw 3 cards.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=5}}
return card

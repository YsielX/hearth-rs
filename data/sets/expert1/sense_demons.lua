local card={api_version=1,id="EX1_317",name="Sense Demons",text="Draw 2 Demons\nfrom your deck.",set="EXPERT1",type="spell",class="warlock",rarity="common",spell_school="shadow",cost=3}
local function demon(ctx,e)for _,t in ipairs(ctx:card_definition(ctx:entity(e).card_id).tags or{})do if t=="demon"or t=="all"then return true end end return false end
local function next_one(ctx,self)local r={};for _,e in ipairs(ctx:deck(ctx:controller(self)))do if demon(ctx,e)then r[#r+1]=e end end;if(ctx:get_data(self,"demons_left")or 0)>0 and #r>0 then ctx:random_entity(r,"sense_demon")end end
function card.on_play(ctx,self)ctx:set_data(self,"demons_left",2);next_one(ctx,self)end
function card.sense_demon(ctx,self,e)ctx:draw_entity(ctx:controller(self),e);ctx:set_data(self,"demons_left",ctx:get_data(self,"demons_left")-1);ctx:continue_with("sense_continue")end
function card.sense_continue(ctx,self)next_one(ctx,self)end
return card

local card={api_version=1,id="LOOT_314",name="Grizzled Guardian",text="<b>Taunt</b>\n<b>Deathrattle:</b> <b>Recruit</b> 2 minions that cost (4) or less.",set="LOOTAPALOOZA",type="minion",class="druid",rarity="rare",cost=8,attack=3,health=5,tags={"beast"},keywords={"taunt","deathrattle"}}
local function next_one(ctx,self)local p=ctx:controller(self);if(ctx:get_data(self,"guardian_left")or 0)<=0 or #ctx:board(p)>=7 then return end;local pool={};for _,e in ipairs(ctx:deck(p))do local x=ctx:entity(e);if x.type=="minion"and x.cost<=4 then pool[#pool+1]=e end end;if #pool>0 then ctx:random_entity(pool,"guardian_recruit")end end
function card.on_deathrattle(ctx,self)ctx:set_data(self,"guardian_left",2);next_one(ctx,self)end
function card.guardian_recruit(ctx,self,e)ctx:recruit(ctx:controller(self),e);ctx:set_data(self,"guardian_left",ctx:get_data(self,"guardian_left")-1);ctx:continue_with("guardian_continue")end
function card.guardian_continue(ctx,self)next_one(ctx,self)end
return card

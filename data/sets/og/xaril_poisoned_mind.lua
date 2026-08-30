local toxins={"OG_080b","OG_080c","OG_080d","OG_080e","OG_080f"}
local function friendly(ctx,self)local p=ctx:controller(self);local r={};for _,m in ipairs(ctx:friendly_minions(self))do r[#r+1]=m end return r end
local card={api_version=1,id="OG_080",name="Xaril, Poisoned Mind",text="<b>Battlecry and Deathrattle:</b> Add a random Toxin card to your hand.",set="OG",type="minion",class="rogue",rarity="legendary",cost=4,attack=3,health=2,keywords={"battlecry","deathrattle"}}
local function toxin(ctx)ctx:random_value(toxins,"receive_toxin")end
card.on_battlecry=toxin;card.on_deathrattle=toxin
function card.receive_toxin(ctx,self,id)cardlib.effects.give_card(ctx, ctx:controller(self),id)end
card.tokens={
 {id="OG_080b",name="Kingsblood Toxin",text="Draw a card.",set="OG",type="spell",class="rogue",spell_school="nature",cost=1,on_play=function(ctx,self)ctx:draw(ctx:controller(self),1)end},
 {id="OG_080c",name="Bloodthistle Toxin",text="Return a friendly minion to your hand.\nIt costs (2) less.",set="OG",type="spell",class="rogue",spell_school="nature",cost=1,target_mode="required",targets=friendly,on_play=function(ctx,self,target)ctx:move(target,"hand");cardlib.effects.modify(ctx, target,{stat="cost",operation="add",value=-2})end},
 {id="OG_080d",name="Briarthorn Toxin",text="Give a minion +3 Attack.",set="OG",type="spell",class="rogue",spell_school="nature",cost=1,target_mode="required",targets=function(ctx)return ctx:minions()end,on_play=function(ctx,self,target)cardlib.effects.buff(ctx, target,3,0)end},
 {id="OG_080e",name="Fadeleaf Toxin",text="Give a friendly minion <b>Stealth</b> until your next turn.",set="OG",type="spell",class="rogue",spell_school="shadow",cost=1,target_mode="required",targets=friendly,on_play=function(ctx,self,target)ctx:set_data(self,"fade_target",target);cardlib.effects.grant_keyword(ctx, target,"stealth")end,triggers={{event="turn_started",timing="after",active_zones={"graveyard"},condition=function(ctx,self,event)return event.player==ctx:controller(self) and ctx:get_data(self,"fade_target")~=0 end,effect=function(ctx,self)local t=ctx:get_data(self,"fade_target");ctx:remove_enchantments_from(t,self);ctx:set_data(self,"fade_target",0)end}}},
 {id="OG_080f",name="Firebloom Toxin",text="Deal $2 damage.",set="OG",type="spell",class="rogue",spell_school="fire",cost=1,target_mode="required",targets=function(ctx)return ctx:characters()end,on_play=function(ctx,self,target)cardlib.effects.damage(ctx, target,2)end},
}
return card

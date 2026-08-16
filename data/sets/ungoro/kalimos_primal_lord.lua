local card={api_version=1,id="UNG_211",name="Kalimos, Primal Lord",text="[x]<b>Battlecry:</b> If you played an\nElemental last turn, cast an\nElemental Invocation.",set="UNGORO",type="minion",class="shaman",rarity="legendary",cost=7,attack=7,health=7,tags={"elemental"},keywords={"battlecry"}}
local invocations={"UNG_211a","UNG_211b","UNG_211c","UNG_211d"}
local function played(ctx,p) for _,id in ipairs(ctx:cards_played_last_turn(p)) do for _,t in ipairs(ctx:card_definition(id).tags or {}) do if t=="elemental" or t=="all" then return true end end end return false end
function card.on_battlecry(ctx,self) if played(ctx,ctx:controller(self)) then ctx:choose_cards(ctx:controller(self),"Choose an Elemental Invocation",invocations,"kalimos_chosen") end end
function card.kalimos_chosen(ctx,self,id) ctx:cast_spell(ctx:controller(self),id) end
card.tokens={
 {id="UNG_211a",name="Invocation of Earth",text="Fill your board with 1/1 Elementals.",set="UNGORO",type="spell",class="shaman",collectible=false,cost=0,rules={can_play=function(ctx,self) return #ctx:board(ctx:controller(self))<7 end},on_play=function(ctx,self) local p=ctx:controller(self); for _=1,7-#ctx:board(p) do ctx:summon(p,"UNG_211aa") end end},
 {id="UNG_211aa",name="Stone Elemental",text="",set="UNGORO",type="minion",class="shaman",collectible=false,cost=1,attack=1,health=1,tags={"elemental"}},
 {id="UNG_211b",name="Invocation of Water",text="Restore 12 Health to your hero.",set="UNGORO",type="spell",class="shaman",collectible=false,cost=0,on_play=function(ctx,self) local p=ctx:controller(self);cardlib.effects.heal(ctx, ctx:player(p).hero,12) end},
 {id="UNG_211c",name="Invocation of Fire",text="Deal 6 damage to the enemy hero.",set="UNGORO",type="spell",class="shaman",collectible=false,cost=0,on_play=function(ctx,self) local p=ctx:opponent(ctx:controller(self));cardlib.effects.damage_ignoring_spell_damage(ctx, ctx:player(p).hero,6) end},
 {id="UNG_211d",name="Invocation of Air",text="Deal 3 damage to all enemy minions.",set="UNGORO",type="spell",class="shaman",collectible=false,cost=0,on_play=function(ctx,self) local t={} for _,e in ipairs(ctx:enemy_characters(self)) do if ctx:entity(e).type=="minion" then t[#t+1]={e,3} end end cardlib.effects.damage_batch_ignoring_spell_damage(ctx, t) end},
}
return card

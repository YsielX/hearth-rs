local card={api_version=1,id="LOOT_170",name="Raven Familiar",text="<b>Battlecry:</b> Reveal a spell in each deck. If yours costs more, draw it.",set="LOOTAPALOOZA",type="minion",class="mage",rarity="common",cost=2,attack=2,health=2,tags={"beast"},keywords={"battlecry"}}
local function spells(ctx,p)local r={};for _,e in ipairs(ctx:deck(p))do if ctx:entity(e).type=="spell"then r[#r+1]=e end end;return r end
function card.on_battlecry(ctx,self)local p=spells(ctx,ctx:controller(self));if #p>0 then ctx:random_entity(p,"raven_own_spell")end end
function card.raven_own_spell(ctx,self,e)ctx:set_data(self,"raven_own",e);local p=spells(ctx,ctx:opponent(ctx:controller(self)));if #p>0 then ctx:random_entity(p,"raven_enemy_spell")end end
function card.raven_enemy_spell(ctx,self,e)local own=ctx:get_data(self,"raven_own");if ctx:entity(own).zone=="deck"and ctx:entity(own).cost>ctx:entity(e).cost then ctx:draw_entity(ctx:controller(self),own)end end
return card

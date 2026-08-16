local card={api_version=1,id="ICC_064",name="Blood Razor",text="<b>Battlecry and Deathrattle:</b>\nDeal 1 damage to all minions.",set="ICECROWN",type="weapon",class="warrior",rarity="common",cost=4,attack=2,health=2,keywords={"battlecry","deathrattle"}}
local function sweep(ctx)cardlib.effects.damage_all(ctx, ctx:minions(),1)end
function card.on_battlecry(ctx,self)sweep(ctx)end
function card.on_deathrattle(ctx,self)sweep(ctx)end
return card

local function dragon(ctx, entity)
    for _, tag in ipairs(ctx:entity(entity).tags or {}) do if tag == "dragon" or tag == "all" then return true end end
    local definition=ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags or {}) do if tag == "dragon" or tag == "all" then return true end end
    return false
end
local card={api_version=1,id="LOOT_410",name="Duskbreaker",text="<b>Battlecry:</b> If you're holding a Dragon, deal 3 damage to all other minions.",set="LOOTAPALOOZA",type="minion",class="priest",rarity="rare",cost=4,attack=3,health=3,tags={"dragon"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)
    local held=false;for _,e in ipairs(ctx:hand(ctx:controller(self)))do if dragon(ctx,e)then held=true;break end end
    if held then local targets={};for _,e in ipairs(ctx:minions())do if e~=self then targets[#targets+1]=e end end;ctx:damage_all(targets,3)end
end
return card

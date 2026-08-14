local card = { api_version = 1, id = "UNG_086", name = "Giant Anaconda",
    text = "<b>Taunt</b>\n<b>Deathrattle:</b> Summon\na minion from your hand with 5 or more Attack.",
    set = "UNGORO", type = "minion", class = "druid", rarity = "epic",
    cost = 7, attack = 5, health = 3, tags = { "beast" }, keywords = { "taunt", "deathrattle" } }
function card.on_deathrattle(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" and ctx:entity(entity).attack >= 5 then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "summon_minion") end
end
function card.summon_minion(ctx, self, entity) ctx:summon_from_hand(entity) end
return card

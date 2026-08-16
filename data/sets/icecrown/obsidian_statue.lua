local card = {
    api_version = 1, id = "ICC_214", name = "Obsidian Statue",
    text = "[x]<b>Taunt, Lifesteal</b>\n<b>Deathrattle:</b> Destroy a\n random enemy minion.",
    set = "ICECROWN", type = "minion", class = "priest", rarity = "epic",
    cost = 9, attack = 4, health = 8, keywords = { "taunt", "lifesteal", "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, entity in ipairs(ctx:enemy_characters(self)) do
        if ctx:entity(entity).type == "minion" then pool[#pool + 1] = entity end
    end
    if #pool > 0 then ctx:random_entity(pool, "destroy_statue_victim") end
end

function card.destroy_statue_victim(ctx, self, target) cardlib.effects.destroy(ctx, target) end

return card
